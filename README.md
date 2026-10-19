# EduSense

Micro-services **Rust** et frontend **React**, développés de façon autonome par Claude selon un
processus piloté étape par étape (spec → tests rouges → code → revue indépendante → refactoring).

- Règles et processus : [Instructions.md](Instructions.md) (référence) et [CLAUDE.md](CLAUDE.md) (synthèse)
- Spécifications : [specs/](specs/) · Revues : [reviews/](reviews/) · Décisions : [docs/adr/](docs/adr/README.md)

## Contenu

| Composant | Rôle |
|---|---|
| [services/health](services/health/README.md) | Service squelette : santé, OpenAPI, Problem Details, corrélation |
| [frontend](frontend/README.md) | Application React (Vite) |

## Démarrage rapide

Prérequis : Rust (la toolchain est épinglée dans `rust-toolchain.toml`), Node.js 24, Docker
(optionnel). Le plus simple : ouvrir le dépôt dans le **devcontainer** VS Code, qui installe tout.

```sh
# Tous les services avec Docker
docker compose up --build        # front : http://localhost:8080, health : http://localhost:8081/health

# Sans Docker
cargo run -p health-service      # http://localhost:8080/health
cd frontend && npm ci && npm run dev
```

## Contrôles qualité

Les mêmes contrôles qu'en CI ([.github/workflows/ci.yml](.github/workflows/ci.yml)) :

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo cov                        # couverture, seuil 100 %
cargo mutants --workspace        # mutation testing
cargo deny check && cargo audit
cargo doc --workspace --no-deps

cd frontend
npm run format:check && npm run lint && npm run typecheck
npm run test:coverage            # seuil 100 %
npm run mutation                 # Stryker
npm audit && npm run build
```

Outils cargo requis : `cargo install --locked cargo-llvm-cov cargo-mutants cargo-deny cargo-audit`
et `rustup component add llvm-tools-preview`.

## Configuration GitHub (à faire une fois)

- Ruleset sur `main` (Settings → Rules → Rulesets) :
  - PR obligatoire avant merge ;
  - check requis **`CI OK`** (synthèse de tous les jobs du workflow CI) ;
  - règle **« Require code scanning results »** : outil CodeQL, seuil *Errors* et alertes de
    sécurité *High or higher*. Un job CodeQL réussit même s'il trouve des alertes : seul ce
    réglage rend CodeQL réellement bloquant.
- Activer Dependabot alerts, secret scanning et push protection (Settings → Code security).
