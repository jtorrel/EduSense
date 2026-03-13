use super::AuthProvider;

pub struct GoogleProvider;

impl AuthProvider for GoogleProvider {
    fn name(&self) -> &str {
        "Google"
    }

    fn authenticate(&self, username: &str, _password: &str) -> bool {
        // Mock : Google n'utilise pas de mot de passe local
        // On simule un token Google valide
        username.ends_with("@normandiewebschool.fr")
    }
}
