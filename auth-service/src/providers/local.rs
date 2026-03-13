use super::{AuthProvider, AuthRequest, AuthResult, ErrorCode};
use async_trait::async_trait;

pub struct LocalProvider;

#[async_trait]
impl AuthProvider for LocalProvider {
    fn name(&self) -> &str {
        "Local"
    }

    async fn authenticate(&self, request: AuthRequest) -> AuthResult {
        match request {
            AuthRequest::Password { username, password } => {
                if username == "admin" && password == "secret" {
                    AuthResult::Success { email: username }
                } else {
                    AuthResult::Failure {
                        code: ErrorCode::InvalidCredentials,
                        reason: "Identifiants invalides".to_string(),
                    }
                }
            }
            _ => AuthResult::Failure {
                code: ErrorCode::Unknown,
                reason: format!("{} ne supporte pas ce flux", self.name()),
            },
        }
    }
}
