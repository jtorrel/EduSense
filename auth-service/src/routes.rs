use crate::providers::{AuthProvider, AuthRequest, AuthResult};
use axum::extract::State;
use axum::{extract::Query, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct AuthQuery {
    pub username: String,
    pub password: String,
    pub provider: String,
}

pub async fn authenticate(
    State(providers): State<Arc<Vec<Box<dyn AuthProvider>>>>,
    Query(params): Query<AuthQuery>,
) -> impl IntoResponse {
    let provider = providers
        .iter()
        .find(|p| p.name().to_lowercase() == params.provider.to_lowercase());

    match provider {
        Some(p) => {
            let result = p
                .authenticate(AuthRequest::Password {
                    username: params.username,
                    password: params.password,
                })
                .await;

            match result {
                AuthResult::Success { email } => {
                    (StatusCode::OK, Json(format!("Authentifié : {}", email))).into_response()
                }
                AuthResult::Failure { reason, .. } => {
                    (StatusCode::UNAUTHORIZED, Json(reason)).into_response()
                }
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Résultat inattendu"),
                )
                    .into_response(),
            }
        }
        None => (StatusCode::BAD_REQUEST, Json("Provider inconnu")).into_response(),
    }
}
