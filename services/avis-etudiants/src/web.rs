//! État partagé, erreurs HTTP, et préparation des données pour les templates.

use std::sync::Arc;

use askama::Template;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::cookie::Key;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Europe::Paris;
use sqlx::MySqlPool;

use crate::auth::CurrentUser;
use crate::config::Config;
use crate::db::{EventRow, TicketRow};
use crate::domain::Status;

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: Arc<Config>,
    pub http: reqwest::Client,
    pub key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(s: &AppState) -> Key {
        s.key.clone()
    }
}

// ---------------------------------------------------------------- erreurs

#[derive(Debug)]
pub enum AppError {
    NotFound,
    Forbidden,
    BadRequest(String),
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Internal(format!("base de données : {e}"))
    }
}

impl From<askama::Error> for AppError {
    fn from(e: askama::Error) -> Self {
        AppError::Internal(format!("template : {e}"))
    }
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorPage<'a> {
    nav: Option<Nav>,
    title: &'a str,
    message: &'a str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (code, title, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Introuvable", "Cette page n'existe pas."),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "Accès refusé",
                "Vous n'avez pas les droits pour cette action.",
            ),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "Requête invalide", m.as_str()),
            AppError::Internal(m) => {
                tracing::error!("{m}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Erreur",
                    "Une erreur est survenue. Réessayez dans un instant.",
                )
            }
        };
        let page = ErrorPage { nav: None, title, message };
        match page.render() {
            Ok(html) => (code, Html(html)).into_response(),
            Err(_) => (code, message.to_string()).into_response(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub fn render<T: Template>(t: &T) -> AppResult<Response> {
    Ok(Html(t.render()?).into_response())
}

pub fn check_csrf(user: &CurrentUser, token: &str) -> AppResult<()> {
    // Comparaison en temps constant.
    let a = user.csrf.as_bytes();
    let b = token.as_bytes();
    let same = a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0;
    if same {
        Ok(())
    } else {
        Err(AppError::BadRequest(
            "Le formulaire a expiré. Rechargez la page et recommencez.".into(),
        ))
    }
}

// ---------------------------------------------------------------- dates

pub fn today_paris() -> NaiveDate {
    Utc::now().with_timezone(&Paris).date_naive()
}

pub fn fmt_date(d: NaiveDate) -> String {
    d.format("%d/%m/%Y").to_string()
}

pub fn fmt_datetime(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Paris).format("%d/%m/%Y à %H:%M").to_string()
}

// ---------------------------------------------------------------- vues

/// Barre de navigation (présente sur les pages connectées).
pub struct Nav {
    pub name: String,
    pub email: String,
    pub is_admin: bool,
    pub is_superadmin: bool,
    pub csrf: String,
}

impl Nav {
    pub fn of(u: &CurrentUser) -> Option<Nav> {
        Some(Nav {
            name: u.name.clone(),
            email: u.email.clone(),
            is_admin: u.role.is_admin(),
            is_superadmin: u.role.is_superadmin(),
            csrf: u.csrf.clone(),
        })
    }
}

pub struct TicketView {
    pub id: u32,
    pub date: String,
    pub category: String,
    pub teacher: String,
    pub rating: u8,
    pub stars: String,
    pub comment: String,
    pub status_label: &'static str,
    pub status_class: &'static str,
    pub author_email: String,
    pub author_name: String,
    pub reply: Option<String>,
    pub replied_by: String,
    pub replied_at: String,
    pub created_at: String,
}

impl TicketView {
    pub fn from_row(r: &TicketRow) -> TicketView {
        let status = r.status();
        TicketView {
            id: r.id,
            date: fmt_date(r.ticket_date),
            category: r.category.clone(),
            teacher: r.teacher.clone(),
            rating: r.rating,
            stars: stars(r.rating),
            comment: r.comment.clone(),
            status_label: status.label(),
            status_class: status.as_str(),
            author_email: r.author_email.clone(),
            author_name: r.author_name.clone().unwrap_or_else(|| r.author_email.clone()),
            reply: r.reply.clone(),
            replied_by: r.replied_by.clone().unwrap_or_default(),
            replied_at: r.replied_at.map(fmt_datetime).unwrap_or_default(),
            created_at: fmt_datetime(r.created_at),
        }
    }
}

pub fn stars(rating: u8) -> String {
    let r = rating.min(5) as usize;
    format!("{}{}", "★".repeat(r), "☆".repeat(5 - r))
}

pub struct EventView {
    pub when: String,
    pub actor: String,
    pub text: String,
}

impl EventView {
    pub fn from_row(e: &EventRow) -> EventView {
        let label = |v: &Option<String>| {
            v.as_deref()
                .and_then(Status::parse)
                .map(Status::label)
                .unwrap_or("?")
                .to_string()
        };
        let text = match e.kind.as_str() {
            "created" => "Ticket créé".to_string(),
            "status" => format!("Statut : {} → {}", label(&e.old_value), label(&e.new_value)),
            "reply" if e.old_value.is_none() => "Réponse envoyée".to_string(),
            "reply" => "Réponse modifiée".to_string(),
            "cancelled" => "Annulé par l'étudiant".to_string(),
            "deleted" => "Supprimé".to_string(),
            other => other.to_string(),
        };
        EventView { when: fmt_datetime(e.created_at), actor: e.actor_email.clone(), text }
    }
}

/// Option d'un <select>.
pub struct Opt {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

pub fn status_options(current: Option<Status>, choices: &[Status]) -> Vec<Opt> {
    choices
        .iter()
        .map(|s| Opt {
            value: s.as_str().to_string(),
            label: s.label().to_string(),
            selected: Some(*s) == current,
        })
        .collect()
}

pub fn category_options(categories: &[String], current: &str) -> Vec<Opt> {
    categories
        .iter()
        .map(|c| Opt { value: c.clone(), label: c.clone(), selected: c == current })
        .collect()
}

/// Messages de confirmation passés en paramètre `?msg=` après une redirection.
pub fn flash_text(code: Option<&str>) -> Option<&'static str> {
    Some(match code? {
        "created" => "Votre ticket a bien été envoyé. Merci !",
        "cancelled" => "Le ticket a été annulé.",
        "status" => "Le statut a été mis à jour.",
        "reply" => "La réponse a été enregistrée.",
        "deleted" => "Le ticket a été supprimé.",
        "admin_added" => "L'admin a été ajouté.",
        "admin_exists" => "Cette personne est déjà admin.",
        "admin_removed" => "L'admin a été retiré.",
        _ => return None,
    })
}

pub fn non_empty(s: &Option<String>) -> Option<&str> {
    s.as_deref().map(str::trim).filter(|s| !s.is_empty())
}
