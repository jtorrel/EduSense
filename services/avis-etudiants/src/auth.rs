//! Connexion Google (OAuth 2.0, flux « authorization code ») et session dans un cookie chiffré.
//!
//! Le jeton d'accès sert uniquement à lire le profil sur l'endpoint userinfo de Google,
//! appelé côté serveur en HTTPS : l'email et `email_verified` viennent donc directement de Google.

use axum::extract::{FromRequestParts, Query, State};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Form;
use axum_extra::extract::cookie::{Cookie, Key, PrivateCookieJar, SameSite};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::db;
use crate::domain::{email_in_domain, normalize_email, Role};
use crate::web::{check_csrf, AppError, AppResult, AppState};

const SESSION_COOKIE: &str = "avis_session";
const STATE_COOKIE: &str = "avis_oauth_state";
const SESSION_HOURS: i64 = 12;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";

#[derive(Serialize, Deserialize)]
struct SessionData {
    email: String,
    name: String,
    csrf: String,
    /// Expiration (timestamp Unix).
    exp: i64,
}

/// Utilisateur connecté. Le rôle est recalculé à chaque requête.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub email: String,
    pub name: String,
    pub csrf: String,
    pub role: Role,
}

pub fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn base_cookie(state: &AppState, name: &'static str, value: String) -> Cookie<'static> {
    Cookie::build((name, value))
        .path("/")
        .http_only(true)
        .secure(state.config.secure_cookies)
        .same_site(SameSite::Lax)
        .build()
}

fn removal(name: &'static str) -> Cookie<'static> {
    Cookie::build((name, "")).path("/").build()
}

async fn resolve_role(state: &AppState, email: &str) -> Result<Role, sqlx::Error> {
    if state.config.superadmins.contains(email) {
        Ok(Role::Superadmin)
    } else if db::is_admin(&state.pool, email).await? {
        Ok(Role::Admin)
    } else {
        Ok(Role::Student)
    }
}

/// Lit la session ; `Ok(None)` si absente, expirée ou hors domaine.
pub async fn current_user(
    state: &AppState,
    jar: &PrivateCookieJar<Key>,
) -> AppResult<Option<CurrentUser>> {
    let Some(cookie) = jar.get(SESSION_COOKIE) else { return Ok(None) };
    let Ok(data) = serde_json::from_str::<SessionData>(cookie.value()) else { return Ok(None) };
    if data.exp < Utc::now().timestamp() || !email_in_domain(&data.email, &state.config.allowed_domain)
    {
        return Ok(None);
    }
    let role = resolve_role(state, &data.email).await?;
    Ok(Some(CurrentUser { email: data.email, name: data.name, csrf: data.csrf, role }))
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let jar = PrivateCookieJar::<Key>::from_request_parts(parts, state)
            .await
            .map_err(IntoResponse::into_response)?;
        match current_user(state, &jar).await {
            Ok(Some(u)) => Ok(u),
            Ok(None) => Err(Redirect::to("/").into_response()),
            Err(e) => Err(e.into_response()),
        }
    }
}

/// Extracteur : utilisateur admin ou superadmin.
pub struct AdminUser(pub CurrentUser);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let u = CurrentUser::from_request_parts(parts, state).await?;
        if u.role.is_admin() {
            Ok(AdminUser(u))
        } else {
            Err(AppError::Forbidden.into_response())
        }
    }
}

/// Extracteur : superadmin uniquement.
pub struct SuperadminUser(pub CurrentUser);

impl FromRequestParts<AppState> for SuperadminUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let u = CurrentUser::from_request_parts(parts, state).await?;
        if u.role.is_superadmin() {
            Ok(SuperadminUser(u))
        } else {
            Err(AppError::Forbidden.into_response())
        }
    }
}

/// Ouvre la session : enregistre l'utilisateur et pose le cookie.
async fn open_session(
    state: &AppState,
    jar: PrivateCookieJar<Key>,
    email: &str,
    name: &str,
) -> AppResult<PrivateCookieJar<Key>> {
    db::upsert_user(&state.pool, email, name).await?;
    let data = SessionData {
        email: email.to_string(),
        name: name.to_string(),
        csrf: random_token(),
        exp: Utc::now().timestamp() + SESSION_HOURS * 3600,
    };
    let value = serde_json::to_string(&data).map_err(|e| AppError::Internal(e.to_string()))?;
    let mut cookie = base_cookie(state, SESSION_COOKIE, value);
    cookie.set_max_age(time_hours(SESSION_HOURS));
    Ok(jar.add(cookie))
}

fn time_hours(h: i64) -> cookie::time::Duration {
    cookie::time::Duration::hours(h)
}

// ---------------------------------------------------------------- routes

pub async fn login(State(state): State<AppState>, jar: PrivateCookieJar<Key>) -> Response {
    let csrf_state = random_token();
    let mut url = url::Url::parse(GOOGLE_AUTH_URL).expect("URL Google valide");
    url.query_pairs_mut()
        .append_pair("client_id", &state.config.google_client_id)
        .append_pair("redirect_uri", &state.config.redirect_uri())
        .append_pair("response_type", "code")
        .append_pair("scope", "openid email profile")
        .append_pair("state", &csrf_state)
        // Indication à Google ; la vérification réelle du domaine se fait au retour.
        .append_pair("hd", &state.config.allowed_domain)
        .append_pair("prompt", "select_account");

    let mut cookie = base_cookie(&state, STATE_COOKIE, csrf_state);
    cookie.set_max_age(cookie::time::Duration::minutes(10));
    (jar.add(cookie), Redirect::to(url.as_str())).into_response()
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct UserInfo {
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
    name: Option<String>,
    hd: Option<String>,
}

pub async fn callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar<Key>,
    Query(q): Query<CallbackQuery>,
) -> AppResult<Response> {
    if q.error.is_some() {
        // L'utilisateur a annulé sur l'écran Google.
        return Ok(Redirect::to("/").into_response());
    }
    let expected = jar.get(STATE_COOKIE).map(|c| c.value().to_string());
    let jar = jar.remove(removal(STATE_COOKIE));
    let (Some(code), Some(got), Some(expected)) = (q.code, q.state, expected) else {
        return Err(AppError::BadRequest("Connexion expirée, veuillez recommencer.".into()));
    };
    if got != expected {
        return Err(AppError::BadRequest("Connexion expirée, veuillez recommencer.".into()));
    }

    let token: TokenResponse = state
        .http
        .post(GOOGLE_TOKEN_URL)
        .form(&[
            ("code", code.as_str()),
            ("client_id", state.config.google_client_id.as_str()),
            ("client_secret", state.config.google_client_secret.as_str()),
            ("redirect_uri", state.config.redirect_uri().as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| AppError::Internal(format!("échange du code Google : {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("réponse token Google : {e}")))?;

    let info: UserInfo = state
        .http
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| AppError::Internal(format!("profil Google : {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("réponse profil Google : {e}")))?;

    let domain = &state.config.allowed_domain;
    let email = info.email.as_deref().map(normalize_email).unwrap_or_default();
    let hd_ok = info.hd.as_deref().map(|h| h.eq_ignore_ascii_case(domain)).unwrap_or(false);
    if !info.email_verified || !hd_ok || !email_in_domain(&email, domain) {
        tracing::warn!("connexion refusée pour « {email} »");
        return Err(AppError::BadRequest(format!(
            "Seuls les comptes @{domain} peuvent se connecter."
        )));
    }
    let name = info.name.filter(|n| !n.trim().is_empty()).unwrap_or_else(|| email.clone());

    let jar = open_session(&state, jar, &email, &name).await?;
    tracing::info!("connexion de {email}");
    Ok((jar, Redirect::to("/")).into_response())
}

#[derive(Deserialize)]
pub struct CsrfForm {
    pub csrf: String,
}

pub async fn logout(
    user: CurrentUser,
    jar: PrivateCookieJar<Key>,
    Form(f): Form<CsrfForm>,
) -> AppResult<Response> {
    check_csrf(&user, &f.csrf)?;
    Ok((jar.remove(removal(SESSION_COOKIE)), Redirect::to("/")).into_response())
}

// ---------------------------------------------------------------- connexion simulée (dev)

#[cfg(feature = "dev-login")]
#[derive(Deserialize)]
pub struct DevLoginQuery {
    email: String,
}

/// `/dev/login?email=…` — compilé seulement avec la feature `dev-login`,
/// et actif seulement si DEV_LOGIN=1.
#[cfg(feature = "dev-login")]
pub async fn dev_login(
    State(state): State<AppState>,
    jar: PrivateCookieJar<Key>,
    Query(q): Query<DevLoginQuery>,
) -> AppResult<Response> {
    if !state.config.dev_login {
        return Err(AppError::NotFound);
    }
    let email = normalize_email(&q.email);
    if !email_in_domain(&email, &state.config.allowed_domain) {
        return Err(AppError::BadRequest("Adresse hors domaine.".into()));
    }
    let name = email.split('@').next().unwrap_or_default().to_string();
    let jar = open_session(&state, jar, &email, &name).await?;
    Ok((jar, Redirect::to("/")).into_response())
}
