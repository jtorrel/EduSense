//! Point d'entrée du binaire `health-service`.
//!
//! Assemblage uniquement (ADR 0011) : logs, configuration, socket, signal d'arrêt.

use std::error::Error;

use health_service::config::Config;
use health_service::telemetry;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filter = std::env::var("RUST_LOG").ok();
    tracing::subscriber::set_global_default(telemetry::subscriber(filter.as_deref()))?;

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            // Message explicite (Display) dans les logs JSON plutôt que le Debug de `main`.
            tracing::error!(%error, "configuration invalide");
            std::process::exit(1);
        }
    };
    let listener = TcpListener::bind(config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "health-service à l'écoute");

    health_service::serve(listener, shutdown_signal()).await?;
    tracing::info!("health-service arrêté");
    Ok(())
}

/// Se termine à la réception de Ctrl+C ou, sous Unix, de `SIGTERM` (arrêt Docker).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("écoute de Ctrl+C");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("écoute de SIGTERM")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
