# health-service

Service squelette (*walking skeleton*, [ADR 0014](../../docs/adr/0014-squelette-executable.md))
indiquant l'état de santé de l'application. Il sert de **modèle** pour les futurs services :
architecture hexagonale, tests tracés vers les critères d'acceptation, OpenAPI, erreurs RFC 9457,
identifiant de corrélation, configuration par environnement.

Spécification : [specs/etape-00-bootstrap.md](../../specs/etape-00-bootstrap.md).

## API

| Méthode | Chemin | Réponse |
|---|---|---|
| `GET` | `/health` | `200` — `{"status":"ok"}` |
| `GET` | `/api-docs/openapi.json` | `200` — spécification OpenAPI 3.1 |
| `*` | autre | `404` — `application/problem+json` (RFC 9457) |

Chaque réponse porte l'en-tête `x-request-id` : celui de la requête s'il est fourni, sinon un
UUID v4 généré. Il figure aussi dans les logs de la requête.

## Configuration

| Variable | Défaut | Description |
|---|---|---|
| `HEALTH_BIND_ADDR` | `0.0.0.0:8080` | Adresse `ip:port` d'écoute. Valeur invalide : arrêt au démarrage. |
| `RUST_LOG` | `info` | Filtre de logs ([syntaxe `EnvFilter`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)). Logs JSON sur stdout. |

## Structure

```
src/
├── main.rs              assemblage uniquement (exclu de la couverture, ADR 0011)
├── lib.rs               serve() : serveur HTTP avec arrêt gracieux
├── config.rs            chargement de la configuration
├── domain.rs            logique métier pure
├── telemetry.rs         logs structurés
└── adapters/http/       routeur axum, OpenAPI, Problem Details
tests/                   tests d'intégration (routeur, serveur TCP réel)
```

## Lancement local

```sh
cargo run -p health-service
curl -i http://localhost:8080/health

# ou avec Docker (depuis la racine du dépôt)
docker build -f services/health/Dockerfile -t edusense/health .
docker run --rm -p 8081:8080 edusense/health
```

L'arrêt gracieux est déclenché par Ctrl+C ou `SIGTERM` (`docker stop`).
