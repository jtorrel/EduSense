//! Accès à la base. Les décisions de droits passent par `domain` ;
//! les modifications se font en transaction avec verrou de ligne (SELECT … FOR UPDATE).

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{MySql, MySqlPool, QueryBuilder};

use crate::domain::{self, NewTicket, Role, Status, TicketState};

pub const PAGE_SIZE: u32 = 50;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TicketRow {
    pub id: u32,
    pub author_email: String,
    pub author_name: Option<String>,
    pub ticket_date: NaiveDate,
    pub category: String,
    pub teacher: String,
    pub rating: u8,
    pub comment: String,
    pub status: String,
    pub reply: Option<String>,
    pub replied_by: Option<String>,
    pub replied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl TicketRow {
    pub fn status(&self) -> Status {
        // Valeur inconnue en base : on la traite comme supprimée (invisible).
        Status::parse(&self.status).unwrap_or(Status::Supprime)
    }
    pub fn state(&self) -> TicketState<'_> {
        TicketState {
            author_email: &self.author_email,
            status: self.status(),
            has_reply: self.reply.is_some(),
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminRow {
    pub email: String,
    pub display_name: Option<String>,
    pub added_by: String,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EventRow {
    pub actor_email: String,
    pub kind: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Résultat d'une action sur un ticket.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Done,
    NotFound,
    Forbidden,
}

const TICKET_SELECT: &str = "SELECT t.id, t.author_email, u.display_name AS author_name, t.ticket_date, \
     t.category, t.teacher, t.rating, t.comment, t.status, t.reply, t.replied_by, t.replied_at, \
     t.created_at FROM tickets t LEFT JOIN users u ON u.email = t.author_email";

// ---------------------------------------------------------------- utilisateurs & admins

pub async fn upsert_user(pool: &MySqlPool, email: &str, name: &str) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO users (email, display_name, last_login) VALUES (?, ?, CURRENT_TIMESTAMP) \
         ON DUPLICATE KEY UPDATE display_name = VALUES(display_name), last_login = CURRENT_TIMESTAMP",
    )
    .bind(email)
    .bind(name)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn is_admin(pool: &MySqlPool, email: &str) -> sqlx::Result<bool> {
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admins WHERE email = ?")
        .bind(email)
        .fetch_one(pool)
        .await?;
    Ok(n > 0)
}

pub async fn list_admins(pool: &MySqlPool) -> sqlx::Result<Vec<AdminRow>> {
    sqlx::query_as(
        "SELECT a.email, u.display_name, a.added_by, a.added_at FROM admins a \
         LEFT JOIN users u ON u.email = a.email ORDER BY a.email",
    )
    .fetch_all(pool)
    .await
}

/// Renvoie `false` si l'adresse était déjà admin.
pub async fn add_admin(pool: &MySqlPool, email: &str, by: &str) -> sqlx::Result<bool> {
    let r = sqlx::query("INSERT IGNORE INTO admins (email, added_by) VALUES (?, ?)")
        .bind(email)
        .bind(by)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn remove_admin(pool: &MySqlPool, email: &str) -> sqlx::Result<bool> {
    let r = sqlx::query("DELETE FROM admins WHERE email = ?")
        .bind(email)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

// ---------------------------------------------------------------- lecture des tickets

pub async fn get_ticket(pool: &MySqlPool, id: u32) -> sqlx::Result<Option<TicketRow>> {
    let mut q = QueryBuilder::<MySql>::new(TICKET_SELECT);
    q.push(" WHERE t.id = ").push_bind(id);
    q.build_query_as().fetch_optional(pool).await
}

pub async fn list_tickets_for_author(pool: &MySqlPool, email: &str) -> sqlx::Result<Vec<TicketRow>> {
    let mut q = QueryBuilder::<MySql>::new(TICKET_SELECT);
    q.push(" WHERE t.author_email = ")
        .push_bind(email)
        .push(" AND t.status <> 'supprime' ORDER BY t.created_at DESC, t.id DESC");
    q.build_query_as().fetch_all(pool).await
}

#[derive(Debug, Default, Clone)]
pub struct AdminFilter {
    pub status: Option<Status>,
    pub category: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    /// Recherche dans l'email / le nom de l'auteur et l'intervenant.
    pub search: Option<String>,
    pub page: u32,
}

fn push_filter(q: &mut QueryBuilder<MySql>, f: &AdminFilter) {
    match f.status {
        Some(s) => {
            q.push(" WHERE t.status = ").push_bind(s.as_str());
        }
        None => {
            q.push(" WHERE t.status <> 'supprime'");
        }
    }
    if let Some(c) = &f.category {
        q.push(" AND t.category = ").push_bind(c.clone());
    }
    if let Some(d) = f.from {
        q.push(" AND t.ticket_date >= ").push_bind(d);
    }
    if let Some(d) = f.to {
        q.push(" AND t.ticket_date <= ").push_bind(d);
    }
    if let Some(s) = &f.search {
        let like = format!("%{}%", escape_like(s));
        q.push(" AND (t.author_email LIKE ")
            .push_bind(like.clone())
            .push(" OR u.display_name LIKE ")
            .push_bind(like.clone())
            .push(" OR t.teacher LIKE ")
            .push_bind(like)
            .push(")");
    }
}

/// Renvoie la page demandée et le nombre total de tickets correspondant au filtre.
pub async fn list_tickets_admin(
    pool: &MySqlPool,
    f: &AdminFilter,
) -> sqlx::Result<(Vec<TicketRow>, i64)> {
    let mut count = QueryBuilder::<MySql>::new(
        "SELECT COUNT(*) FROM tickets t LEFT JOIN users u ON u.email = t.author_email",
    );
    push_filter(&mut count, f);
    let total: i64 = count.build_query_scalar().fetch_one(pool).await?;

    let mut q = QueryBuilder::<MySql>::new(TICKET_SELECT);
    push_filter(&mut q, f);
    q.push(" ORDER BY t.created_at DESC, t.id DESC LIMIT ")
        .push_bind(PAGE_SIZE)
        .push(" OFFSET ")
        .push_bind(f.page.saturating_sub(1) * PAGE_SIZE);
    let rows = q.build_query_as().fetch_all(pool).await?;
    Ok((rows, total))
}

pub async fn list_events(pool: &MySqlPool, ticket_id: u32) -> sqlx::Result<Vec<EventRow>> {
    sqlx::query_as(
        "SELECT actor_email, kind, old_value, new_value, created_at FROM ticket_events \
         WHERE ticket_id = ? ORDER BY created_at, id",
    )
    .bind(ticket_id)
    .fetch_all(pool)
    .await
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

// ---------------------------------------------------------------- écritures

async fn log_event(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ticket_id: u32,
    actor: &str,
    kind: &str,
    old: Option<&str>,
    new: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO ticket_events (ticket_id, actor_email, kind, old_value, new_value) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(ticket_id)
    .bind(actor)
    .bind(kind)
    .bind(old)
    .bind(new)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn create_ticket(pool: &MySqlPool, author: &str, t: &NewTicket) -> sqlx::Result<u32> {
    let mut tx = pool.begin().await?;
    let r = sqlx::query(
        "INSERT INTO tickets (author_email, ticket_date, category, teacher, rating, comment) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(author)
    .bind(t.date)
    .bind(&t.category)
    .bind(&t.teacher)
    .bind(t.rating)
    .bind(&t.comment)
    .execute(&mut *tx)
    .await?;
    let id = r.last_insert_id() as u32;
    log_event(&mut tx, id, author, "created", None, Some(Status::Ouvert.as_str())).await?;
    tx.commit().await?;
    Ok(id)
}

/// Verrouille le ticket et renvoie (auteur, statut, réponse actuelle).
async fn lock_ticket(
    tx: &mut sqlx::Transaction<'_, MySql>,
    id: u32,
) -> sqlx::Result<Option<(String, Status, Option<String>)>> {
    let row: Option<(String, String, Option<String>)> =
        sqlx::query_as("SELECT author_email, status, reply FROM tickets WHERE id = ? FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut **tx)
            .await?;
    Ok(row.map(|(a, s, r)| (a, Status::parse(&s).unwrap_or(Status::Supprime), r)))
}

pub async fn cancel_ticket(pool: &MySqlPool, id: u32, user: &str) -> sqlx::Result<Outcome> {
    let mut tx = pool.begin().await?;
    let Some((author, status, reply)) = lock_ticket(&mut tx, id).await? else {
        return Ok(Outcome::NotFound);
    };
    let state = TicketState { author_email: &author, status, has_reply: reply.is_some() };
    if status == Status::Supprime || author != user {
        return Ok(Outcome::NotFound);
    }
    if !domain::can_cancel(user, &state) {
        return Ok(Outcome::Forbidden);
    }
    sqlx::query("UPDATE tickets SET status = 'annule' WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    log_event(&mut tx, id, user, "cancelled", Some(status.as_str()), Some("annule")).await?;
    tx.commit().await?;
    Ok(Outcome::Done)
}

pub async fn set_status(
    pool: &MySqlPool,
    id: u32,
    actor: &str,
    role: Role,
    to: Status,
) -> sqlx::Result<Outcome> {
    let mut tx = pool.begin().await?;
    let Some((author, status, reply)) = lock_ticket(&mut tx, id).await? else {
        return Ok(Outcome::NotFound);
    };
    if status == Status::Supprime {
        return Ok(Outcome::NotFound);
    }
    let state = TicketState { author_email: &author, status, has_reply: reply.is_some() };
    if !domain::can_set_status(role, &state, to) {
        return Ok(Outcome::Forbidden);
    }
    if status != to {
        sqlx::query("UPDATE tickets SET status = ? WHERE id = ?")
            .bind(to.as_str())
            .bind(id)
            .execute(&mut *tx)
            .await?;
        log_event(&mut tx, id, actor, "status", Some(status.as_str()), Some(to.as_str())).await?;
    }
    tx.commit().await?;
    Ok(Outcome::Done)
}

pub async fn set_reply(
    pool: &MySqlPool,
    id: u32,
    actor: &str,
    role: Role,
    reply: &str,
) -> sqlx::Result<Outcome> {
    let mut tx = pool.begin().await?;
    let Some((author, status, old)) = lock_ticket(&mut tx, id).await? else {
        return Ok(Outcome::NotFound);
    };
    if status == Status::Supprime {
        return Ok(Outcome::NotFound);
    }
    let state = TicketState { author_email: &author, status, has_reply: old.is_some() };
    if !domain::can_reply(role, &state) {
        return Ok(Outcome::Forbidden);
    }
    if old.as_deref() != Some(reply) {
        sqlx::query(
            "UPDATE tickets SET reply = ?, replied_by = ?, replied_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(reply)
        .bind(actor)
        .bind(id)
        .execute(&mut *tx)
        .await?;
        log_event(&mut tx, id, actor, "reply", old.as_deref(), Some(reply)).await?;
    }
    tx.commit().await?;
    Ok(Outcome::Done)
}

pub async fn delete_ticket(pool: &MySqlPool, id: u32, actor: &str, role: Role) -> sqlx::Result<Outcome> {
    let mut tx = pool.begin().await?;
    let Some((author, status, reply)) = lock_ticket(&mut tx, id).await? else {
        return Ok(Outcome::NotFound);
    };
    if status == Status::Supprime {
        return Ok(Outcome::NotFound);
    }
    let state = TicketState { author_email: &author, status, has_reply: reply.is_some() };
    if !domain::can_delete(role, &state) {
        return Ok(Outcome::Forbidden);
    }
    sqlx::query("UPDATE tickets SET status = 'supprime' WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    log_event(&mut tx, id, actor, "deleted", Some(status.as_str()), Some("supprime")).await?;
    tx.commit().await?;
    Ok(Outcome::Done)
}
