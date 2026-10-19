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

use axum::body::Body;
use axum::http::{HeaderName, Request};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::{Level, Span};
use utoipa::{OpenApi, ToSchema};

use crate::domain::{self, HealthStatus};

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
pub fn router() -> Router {
    let request_id = HeaderName::from_static(REQUEST_ID_HEADER);

    // Le dernier `layer` ajouté est le plus externe : l'identifiant est posé en premier,
    // puis visible dans le span de trace, puis recopié dans la réponse.
    Router::new()
        .route("/health", get(health))
        .route(OPENAPI_PATH, get(openapi))
        .fallback(problem::not_found)
        .layer(PropagateRequestIdLayer::new(request_id.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(request_span)
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(SetRequestIdLayer::new(request_id, MakeRequestUuid))
}

/// Span de trace d'une requête : méthode, chemin (sans la query, qui peut contenir des
/// données sensibles) et identifiant de corrélation.
fn request_span(request: &Request<Body>) -> Span {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    tracing::info_span!(
        "request",
        method = %request.method(),
        path = %request.uri().path(),
        request_id = %request_id,
    )
}

/// Sert la spécification OpenAPI du service.
async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Renvoie l'état de santé du service.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses((status = 200, description = "Service opérationnel", body = HealthResponse))
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: domain::check(),
    })
}
