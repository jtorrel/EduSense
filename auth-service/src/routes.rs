use crate::providers::AuthProvider;
use axum::{extract::Query, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;

// La structure qui représente les paramètres de la requête GET
#[derive(Deserialize)]
pub struct AuthQuery {
    pub username: String,
    pub password: String,
    pub provider: String,
}

// Le handler HTTP
pub async fn authenticate(
    Query(params): Query<AuthQuery>,
    axum::extract::State(providers): axum::extract::State<
        Arc<Vec<Box<dyn AuthProvider + Send + Sync>>>,
    >,
) -> impl IntoResponse {
    let provider = providers
        .iter()
        .find(|p| p.name().to_lowercase() == params.provider.to_lowercase());

    match provider {
        Some(p) => {
            if p.authenticate(&params.username, &params.password) {
                (StatusCode::OK, Json("Authentification réussie"))
            } else {
                (StatusCode::UNAUTHORIZED, Json("Identifiants invalides"))
            }
        }
        None => (StatusCode::BAD_REQUEST, Json("Provider inconnu")),
    }
}
