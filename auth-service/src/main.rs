mod providers;
mod routes;

use axum::{routing::get, Router};
use providers::{google::GoogleProvider, local::LocalProvider, saml::SamlProvider, AuthProvider};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Construction de la liste des providers
    let providers: Vec<Box<dyn AuthProvider + Send + Sync>> = vec![
        Box::new(LocalProvider),
        Box::new(GoogleProvider),
        Box::new(SamlProvider),
    ];

    // On enveloppe dans un Arc pour partager entre les threads
    let shared_providers = Arc::new(providers);

    // Construction du routeur
    let app = Router::new()
        .route("/auth", get(routes::authenticate))
        .with_state(shared_providers);

    // Démarrage du serveur
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Serveur démarré sur http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
