//! Configuration du service, lue depuis les variables d'environnement (ADR 0007).
//!
//! | Variable | Défaut | Description |
//! |---|---|---|
//! | `HEALTH_BIND_ADDR` | `0.0.0.0:8080` | Adresse et port d'écoute HTTP. |

use std::net::SocketAddr;

/// Nom de la variable d'environnement contenant l'adresse d'écoute.
pub const BIND_ADDR_VAR: &str = "HEALTH_BIND_ADDR";

/// Adresse d'écoute utilisée lorsque [`BIND_ADDR_VAR`] est absente.
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";

/// Configuration validée du service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Adresse et port sur lesquels le serveur HTTP écoute.
    pub bind_addr: SocketAddr,
}

/// Erreur de chargement de la configuration.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    /// La valeur de [`BIND_ADDR_VAR`] n'est pas une adresse `ip:port` valide.
    #[error("{BIND_ADDR_VAR} invalide : « {value} » n'est pas une adresse ip:port")]
    InvalidBindAddr {
        /// Valeur rejetée.
        value: String,
    },
}

impl Config {
    /// Charge la configuration depuis les variables d'environnement du processus.
    ///
    /// # Errors
    ///
    /// Voir [`Config::from_lookup`].
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Charge la configuration à partir d'une fonction de lecture de variables.
    ///
    /// Cette forme permet de tester le chargement sans modifier l'environnement du processus.
    ///
    /// # Errors
    ///
    /// Renvoie [`ConfigError::InvalidBindAddr`] si l'adresse d'écoute est invalide.
    ///
    /// # Examples
    ///
    /// ```
    /// use health_service::config::Config;
    ///
    /// let config = Config::from_lookup(|_| None).unwrap();
    /// assert_eq!(config.bind_addr.to_string(), "0.0.0.0:8080");
    ///
    /// let config = Config::from_lookup(|_| Some("127.0.0.1:3000".into())).unwrap();
    /// assert_eq!(config.bind_addr.port(), 3000);
    /// ```
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let _ = lookup;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup_with(value: Option<&'static str>) -> impl Fn(&str) -> Option<String> {
        move |name| {
            assert_eq!(name, BIND_ADDR_VAR, "seule {BIND_ADDR_VAR} doit être lue");
            value.map(str::to_owned)
        }
    }

    /// CA-05 — variable absente : adresse par défaut `0.0.0.0:8080`.
    #[test]
    fn ca05_uses_default_bind_addr_when_variable_is_absent() {
        let config = Config::from_lookup(lookup_with(None)).unwrap();
        assert_eq!(config.bind_addr, "0.0.0.0:8080".parse().unwrap());
    }

    /// CA-05 — adresse valide fournie : elle est utilisée (IPv4 et IPv6).
    #[test]
    fn ca05_uses_provided_bind_addr() {
        let config = Config::from_lookup(lookup_with(Some("127.0.0.1:3000"))).unwrap();
        assert_eq!(config.bind_addr, "127.0.0.1:3000".parse().unwrap());

        let config = Config::from_lookup(lookup_with(Some("[::1]:9000"))).unwrap();
        assert_eq!(config.bind_addr, "[::1]:9000".parse().unwrap());
    }

    /// CA-05 — valeur invalide : erreur explicite citant la variable et la valeur.
    #[test]
    fn ca05_rejects_invalid_bind_addr() {
        for value in ["", "localhost", "127.0.0.1", "127.0.0.1:99999", "not an address"] {
            let error = Config::from_lookup(lookup_with(Some(value))).unwrap_err();
            assert_eq!(
                error,
                ConfigError::InvalidBindAddr {
                    value: value.to_owned()
                }
            );
            let message = error.to_string();
            assert!(message.contains(BIND_ADDR_VAR), "{message}");
            assert!(message.contains(&format!("« {value} »")), "{message}");
        }
    }

    /// CA-05 — `from_env` lit l'environnement du processus.
    #[test]
    fn ca05_from_env_reads_process_environment() {
        let expected = Config::from_lookup(|name| std::env::var(name).ok());
        assert_eq!(Config::from_env(), expected);
    }
}
