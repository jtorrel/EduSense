# ADR 0004 — Erreurs typées (thiserror) et Problem Details (RFC 9457)

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

Les erreurs doivent être exploitables côté client sans fuite d'information interne.

## Décision

Erreurs de domaine typées avec **thiserror**. À la frontière HTTP, conversion en réponses **RFC 9457** (`application/problem+json`, champs `type`, `title`, `status`, `detail` optionnel). Les causes internes sont journalisées, jamais renvoyées. Le type Problem Details vit dans le service tant qu'il n'y en a qu'un, puis sera extrait dans un crate partagé de `crates/`.

## Conséquences

Format d'erreur homogène entre services ; mapping explicite domaine → HTTP à écrire et tester.
