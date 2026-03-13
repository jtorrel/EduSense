use super::{AuthProvider, AuthRequest, AuthResult, ErrorCode};
use async_trait::async_trait;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use oauth2::{
    reqwest::async_http_client, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret,
    CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, StandardRevocableToken,
    TokenUrl,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Champs additionnels de la réponse token Google
#[derive(Debug, Deserialize, Serialize)]
pub struct GoogleExtraFields {
    pub id_token: String,
}

impl oauth2::ExtraTokenFields for GoogleExtraFields {}

// Type spécialisé pour le client OAuth Google
type GoogleOAuthClient = Client<
    oauth2::basic::BasicErrorResponse,
    oauth2::StandardTokenResponse<GoogleExtraFields, oauth2::basic::BasicTokenType>,
    oauth2::basic::BasicTokenType,
    oauth2::basic::BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    oauth2::basic::BasicRevocationErrorResponse,
>;

pub struct GoogleProvider {
    client: GoogleOAuthClient,
}

// Les données qu'on extrait du JWT Google
#[derive(Deserialize)]
pub struct GoogleClaims {
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
}

impl GoogleProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        let client = GoogleOAuthClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .expect("URL d'autorisation invalide"),
            Some(
                TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                    .expect("URL de token invalide"),
            ),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri).expect("Redirect URI invalide"));

        GoogleProvider { client }
    }

    pub fn authorization_url(&self) -> (String, CsrfToken, String) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        (
            auth_url.to_string(),
            csrf_token,
            pkce_verifier.secret().to_string(),
        )
    }

    pub async fn exchange_code(
        &self,
        code: String,
        pkce_verifier: String,
    ) -> Result<String, String> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
            .request_async(async_http_client)
            .await
            .map_err(|e| format!("Erreur échange token : {}", e))?;

        Ok(token_response.extra_fields().id_token.clone())
    }

    pub async fn verify_id_token(&self, id_token: &str) -> Result<GoogleClaims, String> {
        let header = decode_header(id_token).map_err(|e| format!("JWT header invalide : {}", e))?;

        let kid = header.kid.ok_or("kid absent du JWT header")?;

        let jwks: Value = reqwest::get("https://www.googleapis.com/oauth2/v3/certs")
            .await
            .map_err(|e| format!("Impossible de récupérer les clés Google : {}", e))?
            .json()
            .await
            .map_err(|e| format!("Réponse clés Google invalide : {}", e))?;

        let key = jwks["keys"]
            .as_array()
            .ok_or("Format JWKS invalide")?
            .iter()
            .find(|k| k["kid"].as_str() == Some(&kid))
            .ok_or("Clé publique introuvable pour ce kid")?;

        let n = key["n"].as_str().ok_or("Composante n manquante")?;
        let e = key["e"].as_str().ok_or("Composante e manquante")?;

        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|e| format!("Clé RSA invalide : {}", e))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation
            .set_audience(&[std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID manquant")]);

        let token_data = decode::<GoogleClaims>(id_token, &decoding_key, &validation)
            .map_err(|e| format!("JWT invalide : {}", e))?;

        if !token_data.claims.email_verified {
            return Err("Email non vérifié".to_string());
        }

        Ok(token_data.claims)
    }
}

#[async_trait]
impl AuthProvider for GoogleProvider {
    fn name(&self) -> &str {
        "Google"
    }

    async fn authenticate(&self, request: AuthRequest) -> AuthResult {
        match request {
            AuthRequest::OAuthInit => {
                let (url, csrf_token, pkce_verifier) = self.authorization_url();
                AuthResult::Redirect {
                    url,
                    state: csrf_token.secret().to_string(),
                    pkce_verifier,
                }
            }

            AuthRequest::OAuthCallback {
                code,
                state: _,
                pkce_verifier,
            } => {
                let id_token = match self.exchange_code(code, pkce_verifier).await {
                    Ok(token) => token,
                    Err(reason) => {
                        return AuthResult::Failure {
                            code: ErrorCode::NetworkError,
                            reason,
                        }
                    }
                };

                match self.verify_id_token(&id_token).await {
                    Ok(claims) => AuthResult::Success {
                        email: claims.email,
                    },
                    Err(reason) => AuthResult::Failure {
                        code: ErrorCode::InvalidToken,
                        reason,
                    },
                }
            }

            _ => AuthResult::Failure {
                code: ErrorCode::Unknown,
                reason: format!("{} ne supporte pas ce flux", self.name()),
            },
        }
    }
}
