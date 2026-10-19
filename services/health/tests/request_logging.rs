//! Journalisation des requêtes (exigence §6 de la spec, ADR 0006).
//!
//! Test isolé dans son propre binaire : le cache d'intérêt des callsites `tracing` est global,
//! et des tests parallèles exécutés sans subscriber pourraient désactiver les événements observés.

use std::io::Write;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::Request;
use health_service::adapters::http::{REQUEST_ID_HEADER, router};
use tower::ServiceExt;
use tracing::Level;
use tracing_subscriber::fmt::MakeWriter;

/// Collecte en mémoire des logs émis pendant un test.
#[derive(Clone, Default)]
struct LogCapture(Arc<Mutex<Vec<u8>>>);

impl LogCapture {
    fn contents(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

impl Write for LogCapture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for LogCapture {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// CA-04 / exigence §6 — chaque requête est journalisée au niveau INFO avec son `x-request-id`,
/// sans la query string (qui peut contenir des données sensibles).
#[tokio::test]
async fn ca04_request_is_logged_at_info_with_request_id() {
    let capture = LogCapture::default();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(Level::INFO)
        .with_writer(capture.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let request = Request::get("/health?token=secret")
        .header(REQUEST_ID_HEADER, "log-correlation-42")
        .body(Body::empty())
        .unwrap();
    router().oneshot(request).await.unwrap();

    let logs = capture.contents();
    assert!(logs.contains("finished processing request"), "{logs}");
    assert!(
        logs.contains(r#""request_id":"log-correlation-42""#),
        "{logs}"
    );
    assert!(logs.contains(r#""path":"/health""#), "{logs}");
    assert!(!logs.contains("secret"), "{logs}");
}
