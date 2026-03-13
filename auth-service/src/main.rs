mod providers;
mod routes;
mod routes_google;
mod state;

use axum::{routing::get, Router};
use providers::{google::GoogleProvider, local::LocalProvider, saml::SamlProvider, AuthProvider};
use state::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Fichier .env introuvable");

    let providers: Vec<Box<dyn AuthProvider>> =
        vec![Box::new(LocalProvider), Box::new(SamlProvider)];

    let app_state = AppState {
        providers: Arc::new(providers),
        google: Arc::new(GoogleProvider::new(
            std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID manquant"),
            std::env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET manquant"),
            std::env::var("GOOGLE_REDIRECT_URI").expect("GOOGLE_REDIRECT_URI manquant"),
        )),
        oauth_store: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/auth", get(routes::authenticate))
        .route("/auth/google", get(routes_google::google_login))
        .route("/auth/google/callback", get(routes_google::google_callback))
        .with_state(app_state); // ← un seul state

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Serveur démarré sur http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
