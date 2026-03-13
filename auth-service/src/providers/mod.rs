pub mod google;
pub mod local;
pub mod saml;

pub trait AuthProvider {
    fn name(&self) -> &str;
    fn authenticate(&self, username: &str, password: &str) -> bool;
}
