use super::AuthProvider;

pub struct LocalProvider;

impl AuthProvider for LocalProvider {
    fn name(&self) -> &str {
        "Local"
    }

    fn authenticate(&self, username: &str, password: &str) -> bool {
        // Mock : un seul utilisateur codé en dur
        username == "admin" && password == "admin"
    }
}
