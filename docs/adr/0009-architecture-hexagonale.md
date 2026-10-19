# ADR 0009 — Architecture hexagonale légère

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

La logique métier doit être testable unitairement, indépendamment de l'infrastructure.

## Décision

Chaque service est organisé en modules : `domain` (logique pure), `ports` (traits définissant les dépendances du domaine), `adapters` (`http` avec axum, `persistence` avec sqlx). Le domaine et les handlers sont testés avec des mocks des ports (**mockall**) ; les adaptateurs de persistance avec **testcontainers**. `main.rs` ne fait que l'assemblage.

## Conséquences

Couplage faible et tests rapides ; légère verbosité (traits, injection) assumée. Les modules vides ne sont pas créés à l'avance.
