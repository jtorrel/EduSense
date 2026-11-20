use askama::Template;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Form;
use axum_extra::extract::cookie::{Key, PrivateCookieJar};
use serde::Deserialize;

use crate::auth::{current_user, CsrfForm, CurrentUser};
use crate::db::{self, Outcome};
use crate::domain::{self, validate_new_ticket};
use crate::web::{
    category_options, check_csrf, flash_text, render, today_paris, AppError, AppResult, AppState,
    Nav, Opt, TicketView,
};

#[derive(Deserialize)]
pub struct FlashQuery {
    msg: Option<String>,
}

// ---------------------------------------------------------------- accueil

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage {
    nav: Option<Nav>,
    domain: String,
    dev_login: bool,
}

pub async fn home(State(state): State<AppState>, jar: PrivateCookieJar<Key>) -> AppResult<Response> {
    match current_user(&state, &jar).await? {
        Some(u) if u.role.is_admin() => Ok(Redirect::to("/admin/tickets").into_response()),
        Some(_) => Ok(Redirect::to("/tickets").into_response()),
        None => render(&LoginPage {
            nav: None,
            domain: state.config.allowed_domain.clone(),
            dev_login: state.config.dev_login,
        }),
    }
}

// ---------------------------------------------------------------- mes tickets

#[derive(Template)]
#[template(path = "my_tickets.html")]
struct MyTicketsPage {
    nav: Option<Nav>,
    flash: Option<&'static str>,
    tickets: Vec<TicketView>,
}

pub async fn my_tickets(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(q): Query<FlashQuery>,
) -> AppResult<Response> {
    let rows = db::list_tickets_for_author(&state.pool, &user.email).await?;
    render(&MyTicketsPage {
        nav: Nav::of(&user),
        flash: flash_text(q.msg.as_deref()),
        tickets: rows.iter().map(TicketView::from_row).collect(),
    })
}

// ---------------------------------------------------------------- nouveau ticket

#[derive(Template)]
#[template(path = "new_ticket.html")]
struct NewTicketPage {
    nav: Option<Nav>,
    errors: Vec<String>,
    today: String,
    categories: Vec<Opt>,
    ratings: Vec<(u8, bool)>,
    date: String,
    teacher: String,
    comment: String,
}

#[derive(Deserialize, Default)]
pub struct NewTicketForm {
    #[serde(default)]
    csrf: String,
    #[serde(default)]
    date: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    teacher: String,
    #[serde(default)]
    rating: String,
    #[serde(default)]
    comment: String,
}

fn new_ticket_page(state: &AppState, user: &CurrentUser, f: &NewTicketForm, errors: Vec<String>) -> NewTicketPage {
    NewTicketPage {
        nav: Nav::of(user),
        errors,
        today: today_paris().format("%Y-%m-%d").to_string(),
        categories: category_options(&state.config.categories, &f.category),
        ratings: (0..=5u8).map(|i| (i, f.rating == i.to_string())).collect(),
        date: f.date.clone(),
        teacher: f.teacher.clone(),
        comment: f.comment.clone(),
    }
}

pub async fn new_ticket_form(State(state): State<AppState>, user: CurrentUser) -> AppResult<Response> {
    let f = NewTicketForm { date: today_paris().format("%Y-%m-%d").to_string(), ..Default::default() };
    render(&new_ticket_page(&state, &user, &f, vec![]))
}

pub async fn create_ticket(
    State(state): State<AppState>,
    user: CurrentUser,
    Form(f): Form<NewTicketForm>,
) -> AppResult<Response> {
    check_csrf(&user, &f.csrf)?;
    match validate_new_ticket(
        &f.date,
        &f.category,
        &f.teacher,
        &f.rating,
        &f.comment,
        &state.config.categories,
        today_paris(),
    ) {
        Ok(t) => {
            let id = db::create_ticket(&state.pool, &user.email, &t).await?;
            tracing::info!("ticket #{id} créé par {}", user.email);
            Ok(Redirect::to("/tickets?msg=created").into_response())
        }
        Err(errors) => {
            let page = new_ticket_page(&state, &user, &f, errors);
            let mut resp = render(&page)?;
            *resp.status_mut() = axum::http::StatusCode::UNPROCESSABLE_ENTITY;
            Ok(resp)
        }
    }
}

// ---------------------------------------------------------------- détail & annulation

#[derive(Template)]
#[template(path = "ticket.html")]
struct TicketPage {
    nav: Option<Nav>,
    flash: Option<&'static str>,
    t: TicketView,
    can_cancel: bool,
}

pub async fn show_ticket(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<u32>,
    Query(q): Query<FlashQuery>,
) -> AppResult<Response> {
    let row = db::get_ticket(&state.pool, id).await?.ok_or(AppError::NotFound)?;
    // Ici, seul l'auteur voit son ticket (les admins ont leur propre page).
    if row.author_email != user.email || !domain::can_view(user.role, &user.email, &row.state()) {
        return Err(AppError::NotFound);
    }
    render(&TicketPage {
        nav: Nav::of(&user),
        flash: flash_text(q.msg.as_deref()),
        can_cancel: domain::can_cancel(&user.email, &row.state()),
        t: TicketView::from_row(&row),
    })
}

pub async fn cancel_ticket(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<u32>,
    Form(f): Form<CsrfForm>,
) -> AppResult<Response> {
    check_csrf(&user, &f.csrf)?;
    match db::cancel_ticket(&state.pool, id, &user.email).await? {
        Outcome::Done => Ok(Redirect::to(&format!("/tickets/{id}?msg=cancelled")).into_response()),
        Outcome::NotFound => Err(AppError::NotFound),
        Outcome::Forbidden => Err(AppError::BadRequest(
            "Ce ticket est déjà pris en charge : il ne peut plus être annulé.".into(),
        )),
    }
}
