//! Tests de l'API HTTP du service, exécutés sur le routeur sans ouvrir de socket.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use health_service::adapters::http::{OPENAPI_PATH, REQUEST_ID_HEADER, router};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn get(uri: &str) -> Response {
    send(Request::get(uri).body(Body::empty()).unwrap()).await
}

async fn send(request: Request<Body>) -> Response {
    router().oneshot(request).await.unwrap()
}

async fn json_body(response: Response) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// CA-01 — `GET /health` renvoie `200` et `{"status":"ok"}`.
#[tokio::test]
async fn ca01_health_returns_ok() {
    let response = get("/health").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/json");
    assert_eq!(
        json_body(response).await,
        serde_json::json!({"status": "ok"})
    );
}

/// CA-03 — une méthode non supportée sur une route existante renvoie `405` en
/// `application/problem+json`, avec l'en-tête `Allow`.
#[tokio::test]
async fn ca03_unsupported_method_returns_problem_details() {
    for (method, uri) in [("POST", "/health"), ("DELETE", OPENAPI_PATH)] {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        let response = send(request).await;

        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "{method} {uri}"
        );
        assert_eq!(
            response.headers()["content-type"],
            "application/problem+json"
        );
        assert!(
            response.headers()["allow"]
                .to_str()
                .unwrap()
                .contains("GET"),
            "{method} {uri}"
        );
        assert_eq!(
            json_body(response).await,
            serde_json::json!({"type": "about:blank", "title": "Method Not Allowed", "status": 405})
        );
    }
}

/// CA-02 — la spécification OpenAPI 3.1 est servie et décrit `GET /health`.
#[tokio::test]
async fn ca02_openapi_document_describes_health() {
    let response = get(OPENAPI_PATH).await;

    assert_eq!(response.status(), StatusCode::OK);
    let doc = json_body(response).await;
    assert!(
        doc["openapi"].as_str().unwrap().starts_with("3.1"),
        "version OpenAPI : {}",
        doc["openapi"]
    );
    assert_eq!(doc["info"]["title"], "health-service");
    assert!(doc["paths"]["/health"]["get"]["responses"]["200"].is_object());
    assert!(doc["components"]["schemas"]["HealthResponse"].is_object());
    // Les erreurs sont documentées au format Problem Details (base du futur client TS).
    assert_eq!(
        doc["paths"]["/health"]["get"]["responses"]["405"]["content"]["application/problem+json"]["schema"]
            ["$ref"],
        "#/components/schemas/ProblemDetails"
    );
    assert_eq!(
        doc["components"]["schemas"]["ProblemDetails"]["properties"]["status"]["type"],
        "integer"
    );
}

/// CA-03 — une route inconnue renvoie `404` en `application/problem+json`, sans détail interne.
#[tokio::test]
async fn ca03_unknown_route_returns_problem_details() {
    let response = get("/does-not-exist").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response.headers()["content-type"],
        "application/problem+json"
    );
    assert_eq!(
        json_body(response).await,
        serde_json::json!({"type": "about:blank", "title": "Not Found", "status": 404})
    );
}

/// CA-04 — un `x-request-id` fourni est renvoyé à l'identique.
#[tokio::test]
async fn ca04_propagates_provided_request_id() {
    let request = Request::get("/health")
        .header(REQUEST_ID_HEADER, "correlation-123")
        .body(Body::empty())
        .unwrap();
    let response = send(request).await;

    assert_eq!(response.headers()[REQUEST_ID_HEADER], "correlation-123");
}

/// CA-04 — sans `x-request-id`, un UUID est généré (y compris sur les réponses d'erreur).
#[tokio::test]
async fn ca04_generates_request_id_when_absent() {
    for uri in ["/health", "/does-not-exist"] {
        let response = get(uri).await;
        let id = response.headers()[REQUEST_ID_HEADER].to_str().unwrap();
        uuid::Uuid::parse_str(id).unwrap_or_else(|_| panic!("UUID attendu, reçu {id}"));
    }
}

/// CA-04 — un `x-request-id` valide de longueur maximale (128) est conservé.
#[tokio::test]
async fn ca04_accepts_request_id_at_max_length() {
    let id = format!("{}-._", "a".repeat(125));
    assert_eq!(id.len(), 128);
    let request = Request::get("/health")
        .header(REQUEST_ID_HEADER, &id)
        .body(Body::empty())
        .unwrap();

    assert_eq!(
        send(request).await.headers()[REQUEST_ID_HEADER],
        id.as_str()
    );
}

/// CA-04 — un `x-request-id` invalide (trop long, vide, caractères interdits) est remplacé
/// par un UUID généré.
#[tokio::test]
async fn ca04_replaces_invalid_request_id() {
    let too_long = "a".repeat(129);
    let invalid: [&[u8]; 6] = [
        too_long.as_bytes(),
        b"",
        b"with space",
        b"semi;colon",
        b"slash/id",
        "accentué".as_bytes(),
    ];
    for value in invalid {
        let request = Request::get("/health")
            .header(REQUEST_ID_HEADER, value)
            .body(Body::empty())
            .unwrap();
        let response = send(request).await;
        let id = response.headers()[REQUEST_ID_HEADER].to_str().unwrap();
        assert!(
            uuid::Uuid::parse_str(id).is_ok(),
            "valeur {:?} : UUID attendu, reçu {id}",
            String::from_utf8_lossy(value)
        );
    }
}

/// CA-04 — deux requêtes sans en-tête reçoivent des identifiants distincts.
#[tokio::test]
async fn ca04_generated_request_ids_are_unique() {
    let first = get("/health").await.headers()[REQUEST_ID_HEADER].clone();
    let second = get("/health").await.headers()[REQUEST_ID_HEADER].clone();
    assert_ne!(first, second);
}
