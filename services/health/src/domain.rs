//! Domaine du service : état de santé, sans aucune dépendance d'infrastructure.

use serde::Serialize;
use utoipa::ToSchema;

/// État de santé du service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Le service est opérationnel.
    Ok,
}

/// Évalue l'état de santé courant du service.
///
/// Sans dépendance externe à vérifier, le service est opérationnel dès qu'il répond.
///
/// # Examples
///
/// ```
/// use health_service::domain::{check, HealthStatus};
///
/// assert_eq!(check(), HealthStatus::Ok);
/// ```
#[must_use]
pub fn check() -> HealthStatus {
    HealthStatus::Ok
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CA-01 — un service qui répond est opérationnel.
    #[test]
    fn ca01_check_reports_ok() {
        assert_eq!(check(), HealthStatus::Ok);
    }

    /// CA-01 — l'état se sérialise en `"ok"`.
    #[test]
    fn ca01_status_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&HealthStatus::Ok).unwrap(), r#""ok""#);
    }
}
