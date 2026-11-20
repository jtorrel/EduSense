pub mod admin;
pub mod student;

use axum::routing::{get, post};
use axum::Router;

use crate::auth;
use crate::web::AppState;

pub fn router(state: AppState) -> Router {
    let r = Router::new()
        .route("/", get(student::home))
        .route("/healthz", get(|| async { "ok" }))
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        .route("/logout", post(auth::logout))
        // Étudiant (et tout utilisateur connecté)
        .route("/tickets", get(student::my_tickets).post(student::create_ticket))
        .route("/tickets/new", get(student::new_ticket_form))
        .route("/tickets/{id}", get(student::show_ticket))
        .route("/tickets/{id}/cancel", post(student::cancel_ticket))
        // Admin
        .route("/admin/tickets", get(admin::list))
        .route("/admin/tickets/{id}", get(admin::show))
        .route("/admin/tickets/{id}/reply", post(admin::reply))
        .route("/admin/tickets/{id}/status", post(admin::set_status))
        // Superadmin
        .route("/admin/tickets/{id}/delete", post(admin::delete))
        .route("/admin/admins", get(admin::admins).post(admin::add_admin))
        .route("/admin/admins/remove", post(admin::remove_admin));

    #[cfg(feature = "dev-login")]
    let r = r.route("/dev/login", get(auth::dev_login));

    r.fallback(|| async { crate::web::AppError::NotFound }).with_state(state)
}
