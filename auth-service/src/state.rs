use crate::providers::{google::GoogleProvider, AuthProvider};
use axum::extract::FromRef;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// L'état global de l'application
#[derive(Clone)]
pub struct AppState {
    pub providers: Arc<Vec<Box<dyn AuthProvider>>>,
    pub google: Arc<GoogleProvider>,
    pub oauth_store: Arc<Mutex<HashMap<String, String>>>,
}

// Axum sait extraire Arc<Vec<Box<dyn AuthProvider>>> depuis AppState
impl FromRef<AppState> for Arc<Vec<Box<dyn AuthProvider>>> {
    fn from_ref(state: &AppState) -> Self {
        state.providers.clone()
    }
}

// Axum sait extraire Arc<GoogleProvider> depuis AppState
impl FromRef<AppState> for Arc<GoogleProvider> {
    fn from_ref(state: &AppState) -> Self {
        state.google.clone()
    }
}

// Axum sait extraire Arc<Mutex<HashMap>> depuis AppState
impl FromRef<AppState> for Arc<Mutex<HashMap<String, String>>> {
    fn from_ref(state: &AppState) -> Self {
        state.oauth_store.clone()
    }
}
