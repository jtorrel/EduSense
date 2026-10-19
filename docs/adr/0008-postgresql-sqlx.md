# ADR 0008 — PostgreSQL via sqlx, une base par service

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

Les futurs services auront besoin de persistance, avec isolation stricte entre services.

## Décision

**PostgreSQL** par défaut, accédé via **sqlx** (requêtes vérifiées à la compilation, migrations intégrées, mode offline avec le répertoire `.sqlx` versionné pour compiler en CI sans base). `JSONB` pour les données semi-structurées. **MongoDB** uniquement sur besoin justifié par un ADR dédié. **Une base par service** ; aucun service n'accède à la base d'un autre. Adaptateurs de persistance testés contre une vraie base via **testcontainers**.

## Conséquences

Introduit avec le premier service persistant (pas à l'étape 0). Les tests d'intégration nécessitent Docker (disponible en CI).
