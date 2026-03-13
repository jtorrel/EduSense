use super::{AuthProvider, AuthRequest, AuthResult, ErrorCode};
use async_trait::async_trait;

pub struct SamlProvider;

#[async_trait]
impl AuthProvider for SamlProvider {
    fn name(&self) -> &str {
        "SAML"
    }

    async fn authenticate(&self, request: AuthRequest) -> AuthResult {
        match request {
            AuthRequest::SamlAssertion { assertion } => {
                if assertion.contains("@edusense.io") {
                    AuthResult::Success { email: assertion }
                } else {
                    AuthResult::Failure {
                        code: ErrorCode::InvalidCredentials,
                        reason: "Assertion SAML invalide".to_string(),
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
