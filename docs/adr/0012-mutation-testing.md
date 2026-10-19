# ADR 0012 — Mutation testing

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

La couverture ne garantit pas que les tests vérifient réellement le comportement.

## Décision

**Rust** : `cargo-mutants`. Sur les PR, exécution sur le code modifié (`--in-diff`) ; exécution complète en local et sur `main`. **Front** : **Stryker** (runner Vitest) sur `src/` hors exclusions de l'ADR 0011, seuil `break` à 100. Tout mutant survivant est soit tué par un nouveau test, soit justifié (mutant équivalent) dans le rapport de revue ; un mutant équivalent récurrent peut être exclu par configuration, avec justification.

## Conséquences

Tests de meilleure qualité ; temps de CI supplémentaire, limité par l'exécution incrémentale.
