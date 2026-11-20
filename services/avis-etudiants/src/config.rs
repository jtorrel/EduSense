use std::collections::HashSet;

use axum_extra::extract::cookie::Key;

use crate::domain::{email_in_domain, normalize_email};

const DEFAULT_CATEGORIES: &str =
    "Cours;Intervenant;Organisation;Locaux et matériel;Vie étudiante;Autre";

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    /// URL publique du service, sans « / » final (ex. https://avis.normandiewebschool.fr)
    pub public_base_url: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub allowed_domain: String,
    pub superadmins: HashSet<String>,
    pub categories: Vec<String>,
    pub secure_cookies: bool,
    /// Connexion simulée (feature `dev-login` + DEV_LOGIN=1).
    pub dev_login: bool,
}

fn var(name: &str) -> Result<String, String> {
    std::env::var(name)
        .map(|v| v.trim().to_string())
        .ok()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| format!("variable d'environnement manquante : {name}"))
}

fn var_or(name: &str, default: &str) -> String {
    var(name).unwrap_or_else(|_| default.to_string())
}

impl Config {
    pub fn from_env() -> Result<(Config, Key), String> {
        let dev_login = cfg!(feature = "dev-login") && var_or("DEV_LOGIN", "0") == "1";

        let allowed_domain = var_or("ALLOWED_DOMAIN", "normandiewebschool.fr").to_lowercase();
        let public_base_url = var("PUBLIC_BASE_URL")?.trim_end_matches('/').to_string();

        // En mode dev, les identifiants Google peuvent être absents.
        let (google_client_id, google_client_secret) = if dev_login {
            (var_or("GOOGLE_CLIENT_ID", ""), var_or("GOOGLE_CLIENT_SECRET", ""))
        } else {
            (var("GOOGLE_CLIENT_ID")?, var("GOOGLE_CLIENT_SECRET")?)
        };

        let superadmins: HashSet<String> = parse_list(&var_or("SUPERADMINS", ""), ',')
            .into_iter()
            .map(|e| normalize_email(&e))
            .collect();
        for e in &superadmins {
            if !email_in_domain(e, &allowed_domain) {
                return Err(format!("SUPERADMINS : « {e} » n'est pas une adresse @{allowed_domain}"));
            }
        }
        if superadmins.is_empty() {
            tracing::warn!("SUPERADMINS est vide : personne ne pourra gérer les admins");
        }

        let categories = parse_list(&var_or("CATEGORIES", DEFAULT_CATEGORIES), ';');
        if categories.is_empty() {
            return Err("CATEGORIES ne contient aucune catégorie".into());
        }

        let secret = var("SESSION_SECRET")?;
        if secret.len() < 32 {
            return Err("SESSION_SECRET doit contenir au moins 32 caractères".into());
        }
        let key = Key::derive_from(secret.as_bytes());

        let config = Config {
            database_url: var("DATABASE_URL")?,
            bind_addr: var_or("BIND_ADDR", "0.0.0.0:8080"),
            secure_cookies: public_base_url.starts_with("https://"),
            public_base_url,
            google_client_id,
            google_client_secret,
            allowed_domain,
            superadmins,
            categories,
            dev_login,
        };
        Ok((config, key))
    }

    pub fn redirect_uri(&self) -> String {
        format!("{}/auth/callback", self.public_base_url)
    }
}

fn parse_list(raw: &str, sep: char) -> Vec<String> {
    raw.split(sep)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}
