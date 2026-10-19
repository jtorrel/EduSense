//! Adaptateur HTTP : routes axum, documentation OpenAPI et middlewares transverses.
//!
//! | Méthode | Chemin | Description |
//! |---|---|---|
//! | `GET` | `/health` | État de santé du service. |
//! | `GET` | `/api-docs/openapi.json` | Spécification OpenAPI du service. |
//!
//! Toute autre route renvoie `404` au format Problem Details ([`problem`]).
//! Chaque réponse porte l'en-tête [`REQUEST_ID_HEADER`] (ADR 0006).

pub mod problem;

use axum::{Json, Router};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::domain::HealthStatus;

/// En-tête HTTP portant l'identifiant de corrélation de la requête.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Chemin de la spécification OpenAPI.
pub const OPENAPI_PATH: &str = "/api-docs/openapi.json";

/// Corps de la réponse de `GET /health`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct HealthResponse {
    /// État de santé du service.
    pub status: HealthStatus,
}

/// Documentation OpenAPI du service, générée depuis le code (ADR 0003).
#[derive(OpenApi)]
#[openapi(
    info(title = "health-service", description = "Service de santé EduSense"),
    paths(health),
    components(schemas(HealthResponse, HealthStatus, problem::ProblemDetails))
)]
pub struct ApiDoc;

/// Construit le routeur HTTP complet du service, middlewares inclus.
///
/// # Examples
///
/// ```
/// let app: axum::Router = health_service::adapters::http::router();
/// ```
#[must_use]
pub fn router() -> Router {
    todo!()
}

/// Renvoie l'état de santé du service.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses((status = 200, description = "Service opérationnel", body = HealthResponse))
)]
async fn health() -> Json<HealthResponse> {
    todo!()
}
