//! # health-service
//!
//! Service squelette (*walking skeleton*, ADR 0014) exposant l'état de santé de l'application.
//! Il sert de modèle pour les futurs micro-services EduSense :
//!
//! - [`domain`] : logique métier pure ;
//! - [`adapters::http`] : adaptateur HTTP (axum), OpenAPI (utoipa), erreurs RFC 9457 ;
//! - [`config`] : configuration par variables d'environnement ;
//! - [`telemetry`] : logs structurés (`tracing`).
//!
//! Le binaire (`main.rs`) ne fait qu'assembler ces briques et appeler [`serve`].
#![deny(missing_docs)]

pub mod adapters;
pub mod config;
pub mod domain;
pub mod telemetry;

use std::future::Future;

use tokio::net::TcpListener;

/// Sert l'API HTTP du service sur `listener` jusqu'à ce que `shutdown` se termine.
///
/// L'arrêt est gracieux : les requêtes en cours sont menées à terme avant le retour.
///
/// # Errors
///
/// Renvoie une erreur d'entrée/sortie si le serveur ne peut pas accepter de connexions.
///
/// # Examples
///
/// ```
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> std::io::Result<()> {
/// let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
/// // Arrêt immédiat : le serveur démarre puis s'arrête proprement.
/// health_service::serve(listener, async {}).await?;
/// # Ok(())
/// # }
/// ```
pub async fn serve(
    listener: TcpListener,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    axum::serve(listener, adapters::http::router())
        .with_graceful_shutdown(shutdown)
        .await
}
