use crate::providers::google::GoogleProvider;
use crate::providers::{AuthProvider, AuthRequest, AuthResult};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// Le type partagé entre les deux handlers
pub type OAuthStateStore = Arc<Mutex<HashMap<String, String>>>;

// Handler 1 — déclenche le flux OAuth
pub async fn google_login(
    State(provider): State<Arc<GoogleProvider>>,
    State(store): State<OAuthStateStore>,
) -> impl IntoResponse {
    let result = provider.authenticate(AuthRequest::OAuthInit).await;

    match result {
        AuthResult::Redirect {
            url,
            state,
            pkce_verifier,
        } => {
            // On stocke le pkce_verifier associé au state
            let mut store = store.lock().await;
            store.insert(state, pkce_verifier);

            // On redirige l'utilisateur vers Google
            Redirect::temporary(&url).into_response()
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Erreur initialisation OAuth"),
        )
            .into_response(),
    }
}

use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

pub async fn google_callback(
    State(provider): State<Arc<GoogleProvider>>,
    State(store): State<OAuthStateStore>,
    Query(params): Query<CallbackParams>,
) -> impl IntoResponse {
    // Étape 1 — Retrouver et supprimer le pkce_verifier associé au state
    let pkce_verifier = {
        let mut store = store.lock().await;
        store.remove(&params.state)
    };

    // Étape 2 — Vérifier que le state existe bien
    let pkce_verifier = match pkce_verifier {
        Some(v) => v,
        None => {
            return (StatusCode::UNAUTHORIZED, Json("State invalide ou expiré")).into_response()
        }
    };

    // Étape 3 — Échanger le code contre un token et vérifier le JWT
    let result = provider
        .authenticate(AuthRequest::OAuthCallback {
            code: params.code,
            state: params.state,
            pkce_verifier, // ← on passe le verifier récupéré du store
        })
        .await;

    // Étape 4 — Répondre selon le résultat
    match result {
        AuthResult::Success { email } => {
            (StatusCode::OK, Json(format!("Authentifié : {}", email))).into_response()
        }

        AuthResult::Failure { reason, .. } => {
            (StatusCode::UNAUTHORIZED, Json(reason)).into_response()
        }

        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json("Erreur inattendue")).into_response(),
    }
}
