---
description: Phase 2 — implémentation minimale, suite verte, couverture 100 %
argument-hint: <XX-nom de l'étape>
---

Phase 2 (code) de l'étape `$ARGUMENTS`.

1. Implémente le **minimum** pour faire passer les tests de la phase 1, en respectant l'architecture hexagonale (ADR 0009). Ne modifie pas les tests pour les faire passer ; si un test est faux, signale-le.
2. Documente tout le code : rustdoc avec doctests sur les API publiques, `//!` de module, TSDoc côté front, README du service à jour (rôle, API, configuration, lancement local).
3. Vérifie localement, et corrige jusqu'à ce que tout passe :
   - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `cargo cov` (alias `cargo llvm-cov` défini dans `.cargo/config.toml`, seuil 100 %)
   - `cargo doc --workspace --no-deps`, `cargo deny check`
   - front : `npm run lint`, `npm run typecheck`, `npm run test:coverage`, `npm run build`
4. Toute exclusion de couverture nouvelle doit être conforme à l'ADR 0011 (et justifiée).
5. Commit : `feat: <résumé> (étape $ARGUMENTS)`.
