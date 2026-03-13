use super::AuthProvider;

pub struct SamlProvider;

impl AuthProvider for SamlProvider {
    fn name(&self) -> &str {
        "SAML"
    }

    fn authenticate(&self, username: &str, _password: &str) -> bool {
        // Mock : on simule une assertion SAML valide
        username.ends_with("@normandiewebschool.fr")
    }
}
