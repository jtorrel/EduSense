# ADR 0006 — Logs structurés tracing et identifiant de corrélation

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

Les services doivent être observables dès le départ, et les requêtes traçables d'un service à l'autre.

## Décision

Logs structurés via **tracing** / **tracing-subscriber** (format JSON). Chaque requête porte un en-tête `x-request-id` : repris s'il est fourni, sinon généré (UUID v4), propagé dans la réponse et inclus dans le span de requête (`tower-http` : `SetRequestIdLayer`, `PropagateRequestIdLayer`, `TraceLayer`). Aucune donnée sensible (secret, jeton, donnée personnelle) n'est journalisée.

## Conséquences

Corrélation prête pour la communication inter-services ; vigilance requise sur le contenu des champs journalisés.
