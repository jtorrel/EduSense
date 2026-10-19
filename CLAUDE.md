# CLAUDE.md

Règles du projet EduSense pour Claude Code. Synthèse de [Instructions.md](Instructions.md), qui
fait foi en cas de doute. **Le repo est la seule source de vérité** : toute décision doit y être
consignée (spec, ADR, ce fichier).

## Projet

Micro-services **Rust** (workspace Cargo) + frontend **React/TypeScript** (Vite). Objectif :
développement autonome par Claude selon un processus rigoureux piloté par l'utilisateur.

## Règles non négociables

- **TDD** : tests écrits avant le code ; le commit de phase 1 (`test: ...`) doit faire
  **échouer** les tests.
- **Couverture 100 %** (Rust : `cargo llvm-cov`, lignes ; front : Vitest v8, toutes métriques).
  Exclusions uniquement selon l'[ADR 0011](docs/adr/0011-politique-exclusions-couverture.md).
- **Mutation testing** : `cargo mutants` (Rust), Stryker (front). Mutants survivants traités en
  phase 3/4.
- **Documentation** : `#![deny(missing_docs)]` dans chaque crate, doctests ; TSDoc sur tout
  export du front ; README par service ; ADR pour chaque décision structurante.
- **Sécurité** : `unsafe` interdit (sauf ADR), clippy pedantic sans warning, pas de fuite
  d'information dans les erreurs (RFC 9457), pas de données sensibles dans les logs.
- **Traçabilité** : chaque test référence le critère d'acceptation couvert (`ca01_...` ou
  `CA-01` dans le nom / la doc du test).
- **Aucune hypothèse silencieuse** : toute ambiguïté est remontée ou consignée dans la section 8
  de la spec.
- **Pilotage** : on ne passe jamais à l'étape suivante sans merge de la PR par l'utilisateur.

## Processus d'une étape

Spec : `specs/etape-XX-<nom>.md` (modèle : [specs/_template.md](specs/_template.md)).
Branche : `etape/XX-<nom>`. Commits [Conventional Commits](https://www.conventionalcommits.org/).

| Phase | Commande | Commit |
|---|---|---|
| 0. Clarification | `/phase-0-clarif` | `docs(spec): ...` |
| 1. Tests (rouges) | `/phase-1-tests` | `test: ...` |
| 2. Code (vert, 100 %) | `/phase-2-code` | `feat: ...` |
| 3. Revue indépendante | `/phase-3-review` | `review: ...` |
| 4. Refactoring & sécurité | `/phase-4-refactor` | `refactor: ...` |

Puis ouverture d'une PR vers `main`. Le merge vaut validation.

## Arborescence

```
services/<service>/   crate binaire par micro-service (hexagonal : domain / ports / adapters)
crates/               bibliothèques partagées
frontend/             React + Vite + Vitest
specs/  reviews/  docs/adr/
```

## Architecture d'un service (ADR 0009)

- `domain` : logique pure, aucune dépendance d'infrastructure.
- `ports` : traits (repositories…), mockés avec `mockall` dans les tests.
- `adapters` : HTTP (axum) et persistance (sqlx, testée avec `testcontainers`).
- `main.rs` réduit au strict minimum (lecture config, init logs, bind, appel de `serve`) : seul
  fichier exclu de la couverture.

## Stack (ADR 0002 à 0010)

Tokio, axum, REST/JSON, utoipa (OpenAPI), serde, thiserror + Problem Details RFC 9457,
validator, tracing + `x-request-id`, configuration par variables d'environnement, PostgreSQL via
sqlx (une base par service, `.sqlx` versionné). Front : React, TypeScript, Vite, Vitest,
Testing Library, Stryker, ESLint, Prettier.

## Commandes utiles

```sh
# Rust
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --fail-under-lines 100 --ignore-filename-regex 'main\.rs$'
cargo mutants --workspace          # ou --in-diff <fichier.diff>
cargo deny check
cargo doc --workspace --no-deps

# Front (dans frontend/)
npm run lint
npm run format:check
npm run typecheck
npm run test:coverage
npm run mutation
npm run build
```

## Environnement

- Fins de ligne LF (`.gitattributes`). Plateforme cible : Ubuntu (CI) ; devcontainer fourni.
- Le poste de l'utilisateur est sous Windows 11 : les commandes doivent fonctionner dans Git Bash
  comme sous Linux. Docker n'est pas forcément disponible localement — la CI fait foi.
