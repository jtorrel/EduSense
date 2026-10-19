# ADR 0002 — Runtime Tokio et framework web axum

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

Les services sont des API HTTP asynchrones qui doivent être testables sans serveur réel.

## Décision

Runtime **Tokio** ; framework **axum**, avec les middlewares **tower / tower-http** (timeout, CORS, trace, request-id, rate limiting). Les handlers sont testés via `tower::ServiceExt::oneshot` sur le `Router`, sans ouvrir de socket.

## Conséquences

Écosystème standard maintenu par l'équipe Tokio. Tests d'API rapides. Dépendance forte à l'écosystème tower.
