//! Logs structurés (ADR 0006).
//!
//! Les logs sont émis en JSON sur la sortie standard. Le niveau est piloté par la variable
//! `RUST_LOG` (défaut : [`DEFAULT_FILTER`]).

use tracing::Subscriber;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;

/// Filtre de logs appliqué lorsque `RUST_LOG` est absente ou invalide.
pub const DEFAULT_FILTER: &str = "info";

/// Construit le filtre de niveaux de logs.
///
/// `filter` suit la syntaxe de `RUST_LOG` (ex. `info,health_service=debug`) ; `None` ou une
/// valeur invalide appliquent [`DEFAULT_FILTER`].
///
/// # Examples
///
/// ```
/// use health_service::telemetry::env_filter;
///
/// assert_eq!(env_filter(None).to_string(), "info");
/// assert_eq!(env_filter(Some("debug")).to_string(), "debug");
/// ```
#[must_use]
pub fn env_filter(filter: Option<&str>) -> EnvFilter {
    filter
        .and_then(|directives| EnvFilter::try_new(directives).ok())
        .unwrap_or_else(|| EnvFilter::new(DEFAULT_FILTER))
}

/// Construit le *subscriber* `tracing` du service : filtre [`env_filter`] et sortie JSON.
///
/// # Examples
///
/// ```
/// let subscriber = health_service::telemetry::subscriber(None);
/// tracing::subscriber::with_default(subscriber, || tracing::info!("démarrage"));
/// ```
#[must_use]
pub fn subscriber(filter: Option<&str>) -> impl Subscriber + Send + Sync {
    tracing_subscriber::registry()
        .with(env_filter(filter))
        .with(tracing_subscriber::fmt::layer().json())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exigence §6 — sans `RUST_LOG`, les logs sont émis au niveau `info`.
    #[test]
    fn env_filter_defaults_to_info() {
        assert_eq!(env_filter(None).to_string(), DEFAULT_FILTER);
    }

    /// Exigence §6 — des directives `RUST_LOG` valides sont appliquées.
    #[test]
    fn env_filter_uses_valid_directives() {
        assert_eq!(
            env_filter(Some("warn,health_service=debug")).to_string(),
            // EnvFilter réordonne les directives, de la plus spécifique à la plus générale.
            "health_service=debug,warn"
        );
    }

    /// Exigence §6 — des directives invalides ne coupent pas les logs : repli sur `info`.
    #[test]
    fn env_filter_falls_back_to_default_on_invalid_directives() {
        assert_eq!(env_filter(Some("=[")).to_string(), DEFAULT_FILTER);
    }

    /// Exigence §6 — le subscriber du service applique le filtre de niveaux.
    #[test]
    fn subscriber_applies_filter() {
        let subscriber = subscriber(Some("warn"));
        tracing::subscriber::with_default(subscriber, || {
            assert!(tracing::enabled!(tracing::Level::WARN));
            assert!(!tracing::enabled!(tracing::Level::INFO));
        });
    }
}
