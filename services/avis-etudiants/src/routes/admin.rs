use askama::Template;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Form;
use chrono::NaiveDate;
use serde::Deserialize;

use crate::auth::{AdminUser, CsrfForm, SuperadminUser};
use crate::db::{self, AdminFilter, Outcome, PAGE_SIZE};
use crate::domain::{self, email_in_domain, normalize_email, validate_reply, Status};
use crate::web::{
    category_options, check_csrf, flash_text, fmt_datetime, non_empty, render, status_options,
    AppError, AppResult, AppState, EventView, Nav, Opt, TicketView,
};

// ---------------------------------------------------------------- liste

#[derive(Deserialize, Default)]
pub struct ListQuery {
    status: Option<String>,
    category: Option<String>,
    from: Option<String>,
    to: Option<String>,
    q: Option<String>,
    page: Option<u32>,
    msg: Option<String>,
}

pub struct AdminTicketRow {
    pub t: TicketView,
    pub untreated: bool,
}

#[derive(Template)]
#[template(path = "admin_tickets.html")]
struct AdminTicketsPage {
    nav: Option<Nav>,
    flash: Option<&'static str>,
    rows: Vec<AdminTicketRow>,
    total: i64,
    statuses: Vec<Opt>,
    categories: Vec<Opt>,
    from: String,
    to: String,
    q: String,
    page: u32,
    pages: u32,
    prev_url: Option<String>,
    next_url: Option<String>,
}

fn parse_date(s: &Option<String>) -> Option<NaiveDate> {
    non_empty(s).and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
}

fn page_url(q: &ListQuery, page: u32) -> String {
    let mut url = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in [("status", &q.status), ("category", &q.category), ("from", &q.from), ("to", &q.to), ("q", &q.q)] {
        if let Some(v) = non_empty(v) {
            url.append_pair(k, v);
        }
    }
    url.append_pair("page", &page.to_string());
    format!("/admin/tickets?{}", url.finish())
}

pub async fn list(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Query(q): Query<ListQuery>,
) -> AppResult<Response> {
    let status = non_empty(&q.status).and_then(Status::parse).filter(|s| *s != Status::Supprime);
    let category = non_empty(&q.category).map(String::from);
    let filter = AdminFilter {
        status,
        category: category.clone(),
        from: parse_date(&q.from),
        to: parse_date(&q.to),
        search: non_empty(&q.q).map(String::from),
        page: q.page.unwrap_or(1).max(1),
    };
    let (rows, total) = db::list_tickets_admin(&state.pool, &filter).await?;
    let pages = (total.max(0) as u32).div_ceil(PAGE_SIZE);
    let page = filter.page;

    render(&AdminTicketsPage {
        nav: Nav::of(&user),
        flash: flash_text(q.msg.as_deref()),
        rows: rows
            .iter()
            .map(|r| AdminTicketRow { t: TicketView::from_row(r), untreated: domain::is_untreated(&r.state()) })
            .collect(),
        total,
        statuses: status_options(status, &Status::VISIBLE),
        categories: category_options(&state.config.categories, category.as_deref().unwrap_or("")),
        from: filter.from.map(|d| d.to_string()).unwrap_or_default(),
        to: filter.to.map(|d| d.to_string()).unwrap_or_default(),
        q: non_empty(&q.q).unwrap_or_default().to_string(),
        page,
        pages: pages.max(1),
        prev_url: (page > 1).then(|| page_url(&q, page - 1)),
        next_url: (page < pages).then(|| page_url(&q, page + 1)),
    })
}

// ---------------------------------------------------------------- détail

#[derive(Template)]
#[template(path = "admin_ticket.html")]
struct AdminTicketPage {
    nav: Option<Nav>,
    flash: Option<&'static str>,
    error: Option<String>,
    t: TicketView,
    can_reply: bool,
    can_set_status: bool,
    can_delete: bool,
    statuses: Vec<Opt>,
    reply_draft: String,
    events: Vec<EventView>,
}

async fn ticket_page(
    state: &AppState,
    user: &crate::auth::CurrentUser,
    id: u32,
    flash: Option<&'static str>,
    error: Option<String>,
    reply_draft: Option<String>,
) -> AppResult<Response> {
    let row = db::get_ticket(&state.pool, id).await?.ok_or(AppError::NotFound)?;
    let st = row.state();
    if !domain::can_view(user.role, &user.email, &st) {
        return Err(AppError::NotFound);
    }
    let events = db::list_events(&state.pool, id).await?;
    let page = AdminTicketPage {
        nav: Nav::of(user),
        flash,
        error,
        can_reply: domain::can_reply(user.role, &st),
        can_set_status: !st.status.is_terminal(),
        can_delete: domain::can_delete(user.role, &st),
        statuses: status_options(Some(st.status), &Status::ADMIN_SETTABLE),
        reply_draft: reply_draft.unwrap_or_else(|| row.reply.clone().unwrap_or_default()),
        events: events.iter().map(EventView::from_row).collect(),
        t: TicketView::from_row(&row),
    };
    let mut resp = render(&page)?;
    if page.error.is_some() {
        *resp.status_mut() = axum::http::StatusCode::UNPROCESSABLE_ENTITY;
    }
    Ok(resp)
}

#[derive(Deserialize)]
pub struct FlashQuery {
    msg: Option<String>,
}

pub async fn show(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Path(id): Path<u32>,
    Query(q): Query<FlashQuery>,
) -> AppResult<Response> {
    ticket_page(&state, &user, id, flash_text(q.msg.as_deref()), None, None).await
}

fn outcome_redirect(o: Outcome, to: String) -> AppResult<Response> {
    match o {
        Outcome::Done => Ok(Redirect::to(&to).into_response()),
        Outcome::NotFound => Err(AppError::NotFound),
        Outcome::Forbidden => Err(AppError::BadRequest(
            "Action impossible : ce ticket a été annulé ou supprimé.".into(),
        )),
    }
}

#[derive(Deserialize)]
pub struct ReplyForm {
    csrf: String,
    #[serde(default)]
    reply: String,
}

pub async fn reply(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Path(id): Path<u32>,
    Form(f): Form<ReplyForm>,
) -> AppResult<Response> {
    check_csrf(&user, &f.csrf)?;
    let text = match validate_reply(&f.reply) {
        Ok(t) => t,
        Err(e) => return ticket_page(&state, &user, id, None, Some(e), Some(f.reply)).await,
    };
    let o = db::set_reply(&state.pool, id, &user.email, user.role, &text).await?;
    outcome_redirect(o, format!("/admin/tickets/{id}?msg=reply"))
}

#[derive(Deserialize)]
pub struct StatusForm {
    csrf: String,
    status: String,
}

pub async fn set_status(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Path(id): Path<u32>,
    Form(f): Form<StatusForm>,
) -> AppResult<Response> {
    check_csrf(&user, &f.csrf)?;
    let to = Status::parse(&f.status).ok_or_else(|| AppError::BadRequest("Statut inconnu.".into()))?;
    let o = db::set_status(&state.pool, id, &user.email, user.role, to).await?;
    outcome_redirect(o, format!("/admin/tickets/{id}?msg=status"))
}

pub async fn delete(
    State(state): State<AppState>,
    SuperadminUser(user): SuperadminUser,
    Path(id): Path<u32>,
    Form(f): Form<CsrfForm>,
) -> AppResult<Response> {
    check_csrf(&user, &f.csrf)?;
    let o = db::delete_ticket(&state.pool, id, &user.email, user.role).await?;
    if o == Outcome::Done {
        tracing::info!("ticket #{id} supprimé par {}", user.email);
    }
    outcome_redirect(o, "/admin/tickets?msg=deleted".to_string())
}

// ---------------------------------------------------------------- gestion des admins

pub struct AdminView {
    pub email: String,
    pub name: String,
    pub added: String,
}

#[derive(Template)]
#[template(path = "admins.html")]
struct AdminsPage {
    nav: Option<Nav>,
    flash: Option<&'static str>,
    error: Option<String>,
    domain: String,
    superadmins: Vec<String>,
    admins: Vec<AdminView>,
    email: String,
}

async fn admins_page(
    state: &AppState,
    user: &crate::auth::CurrentUser,
    flash: Option<&'static str>,
    error: Option<String>,
    email: String,
) -> AppResult<Response> {
    let rows = db::list_admins(&state.pool).await?;
    let mut superadmins: Vec<String> = state.config.superadmins.iter().cloned().collect();
    superadmins.sort();
    let page = AdminsPage {
        nav: Nav::of(user),
        flash,
        error,
        domain: state.config.allowed_domain.clone(),
        superadmins,
        admins: rows
            .into_iter()
            .map(|a| AdminView {
                name: a.display_name.unwrap_or_default(),
                added: format!("{} par {}", fmt_datetime(a.added_at), a.added_by),
                email: a.email,
            })
            .collect(),
        email,
    };
    let mut resp = render(&page)?;
    if page.error.is_some() {
        *resp.status_mut() = axum::http::StatusCode::UNPROCESSABLE_ENTITY;
    }
    Ok(resp)
}

pub async fn admins(
    State(state): State<AppState>,
    SuperadminUser(user): SuperadminUser,
    Query(q): Query<FlashQuery>,
) -> AppResult<Response> {
    admins_page(&state, &user, flash_text(q.msg.as_deref()), None, String::new()).await
}

#[derive(Deserialize)]
pub struct AdminEmailForm {
    csrf: String,
    #[serde(default)]
    email: String,
}

pub async fn add_admin(
    State(state): State<AppState>,
    SuperadminUser(user): SuperadminUser,
    Form(f): Form<AdminEmailForm>,
) -> AppResult<Response> {
    check_csrf(&user, &f.csrf)?;
    let email = normalize_email(&f.email);
    let domain = &state.config.allowed_domain;
    if !email_in_domain(&email, domain) {
        let msg = format!("Saisissez une adresse @{domain}.");
        return admins_page(&state, &user, None, Some(msg), f.email).await;
    }
    if state.config.superadmins.contains(&email) {
        let msg = "Cette personne est déjà superadmin.".to_string();
        return admins_page(&state, &user, None, Some(msg), f.email).await;
    }
    let added = db::add_admin(&state.pool, &email, &user.email).await?;
    if added {
        tracing::info!("{email} ajouté aux admins par {}", user.email);
    }
    let msg = if added { "admin_added" } else { "admin_exists" };
    Ok(Redirect::to(&format!("/admin/admins?msg={msg}")).into_response())
}

pub async fn remove_admin(
    State(state): State<AppState>,
    SuperadminUser(user): SuperadminUser,
    Form(f): Form<AdminEmailForm>,
) -> AppResult<Response> {
    check_csrf(&user, &f.csrf)?;
    let email = normalize_email(&f.email);
    if db::remove_admin(&state.pool, &email).await? {
        tracing::info!("{email} retiré des admins par {}", user.email);
    }
    Ok(Redirect::to("/admin/admins?msg=admin_removed").into_response())
}
