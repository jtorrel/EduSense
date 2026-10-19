# ADR 0015 — Toolchain et lints Rust

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

La CI et les postes de développement doivent compiler avec le même compilateur et les mêmes règles.

## Décision

Toolchain **stable épinglée** dans `rust-toolchain.toml` (mise à jour volontaire, par PR), édition **2024**. Lints définis au niveau du workspace (`[workspace.lints]`) : `unsafe_code = "forbid"`, `missing_docs = "deny"`, `clippy::pedantic` activé, warnings traités comme erreurs en CI (`-D warnings`). `cargo-deny` vérifie licences (liste blanche permissive), sources (crates.io uniquement), avis de sécurité et doublons.

## Conséquences

Builds reproductibles ; montée de version de Rust explicite.
