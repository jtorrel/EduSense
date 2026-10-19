# ADR 0005 — Validation des entrées avec validator

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

Toute entrée externe doit être validée avant d'atteindre le domaine.

## Décision

Validation déclarative des DTO avec la crate **validator** (`#[derive(Validate)]`), exécutée dans un extracteur axum dédié. Un échec produit une réponse `422` Problem Details listant les champs invalides, sans écho de valeurs sensibles. Les invariants métier restent garantis par les types du domaine.

## Conséquences

Validation centralisée et testable ; double niveau (DTO + domaine) assumé. Introduit avec le premier endpoint acceptant un payload.
