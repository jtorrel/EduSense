pub mod google;
pub mod local;
pub mod saml;

use async_trait::async_trait;

pub enum AuthRequest {
    Password {
        username: String,
        password: String,
    },
    OAuthInit,
    OAuthCallback {
        code: String,
        state: String,
        pkce_verifier: String,
    },
    SamlAssertion {
        assertion: String,
    },
    Token {
        token: String,
    },
}

pub enum ErrorCode {
    InvalidCredentials,
    ProviderUnavailable,
    InvalidToken,
    InvalidState,
    NetworkError,
    Unknown,
}

pub enum AuthResult {
    Success {
        email: String,
    },
    Redirect {
        url: String,
        state: String,
        pkce_verifier: String,
    },
    Failure {
        code: ErrorCode,
        reason: String,
    },
}

#[async_trait] // ← résout le problème dyn + async
pub trait AuthProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn authenticate(&self, request: AuthRequest) -> AuthResult;
}
