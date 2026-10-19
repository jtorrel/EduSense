//! Réponses d'erreur au format Problem Details (RFC 9457, ADR 0004).
//!
//! Seules des informations destinées au client y figurent : aucune cause interne.

use axum::Json;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use serde::{Serialize, Serializer};
use utoipa::ToSchema;

/// Type de média des réponses Problem Details.
pub const PROBLEM_JSON: &str = "application/problem+json";

/// Corps d'une réponse d'erreur RFC 9457.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ProblemDetails {
    /// URI identifiant le type de problème (`about:blank` pour une erreur HTTP générique).
    #[serde(rename = "type")]
    pub problem_type: String,
    /// Résumé lisible du type de problème.
    pub title: String,
    /// Code de statut HTTP, identique à celui de la réponse (sérialisé en entier).
    #[serde(serialize_with = "serialize_status")]
    #[schema(value_type = u16)]
    pub status: StatusCode,
    /// Explication propre à cette occurrence, destinée au client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ProblemDetails {
    /// Construit un problème générique (`about:blank`) pour un statut HTTP donné, son titre
    /// étant la raison standard du statut.
    ///
    /// # Examples
    ///
    /// ```
    /// use axum::http::StatusCode;
    /// use health_service::adapters::http::problem::ProblemDetails;
    ///
    /// let problem = ProblemDetails::from_status(StatusCode::NOT_FOUND);
    /// assert_eq!(problem.title, "Not Found");
    /// assert_eq!(problem.status, StatusCode::NOT_FOUND);
    /// ```
    #[must_use]
    pub fn from_status(status: StatusCode) -> Self {
        Self {
            problem_type: "about:blank".to_owned(),
            title: status.canonical_reason().unwrap_or("Unknown").to_owned(),
            status,
            detail: None,
        }
    }
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        (self.status, [(CONTENT_TYPE, PROBLEM_JSON)], Json(self)).into_response()
    }
}

/// Gestionnaire de repli : toute route inconnue produit un `404` Problem Details.
pub async fn not_found() -> ProblemDetails {
    ProblemDetails::from_status(StatusCode::NOT_FOUND)
}

/// Gestionnaire des méthodes non supportées sur une route existante : `405` Problem Details.
///
/// axum conserve l'en-tête `Allow` listant les méthodes acceptées.
pub async fn method_not_allowed() -> ProblemDetails {
    ProblemDetails::from_status(StatusCode::METHOD_NOT_ALLOWED)
}

/// Sérialise un statut HTTP en entier (membre `status` de la RFC 9457).
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "signature imposée par serde"
)]
fn serialize_status<S: Serializer>(status: &StatusCode, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u16(status.as_u16())
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    /// CA-03 — un problème générique reprend le statut et sa raison standard.
    #[test]
    fn ca03_from_status_builds_generic_problem() {
        assert_eq!(
            ProblemDetails::from_status(StatusCode::NOT_FOUND),
            ProblemDetails {
                problem_type: "about:blank".to_owned(),
                title: "Not Found".to_owned(),
                status: StatusCode::NOT_FOUND,
                detail: None,
            }
        );
        assert_eq!(
            ProblemDetails::from_status(StatusCode::INTERNAL_SERVER_ERROR).title,
            "Internal Server Error"
        );
    }

    /// CA-03 — la réponse porte le statut, le type de média RFC 9457 et le corps JSON.
    #[tokio::test]
    async fn ca03_into_response_uses_problem_json() {
        let problem = ProblemDetails {
            problem_type: "https://edusense.example/problems/test".to_owned(),
            title: "Test".to_owned(),
            status: StatusCode::UNPROCESSABLE_ENTITY,
            detail: Some("détail client".to_owned()),
        };
        let response = problem.into_response();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response.headers()["content-type"], PROBLEM_JSON);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "type": "https://edusense.example/problems/test",
                "title": "Test",
                "status": 422,
                "detail": "détail client",
            })
        );
    }
}
