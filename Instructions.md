# Instructions du projet

> Document de référence du projet : règles, processus de développement et outillage.
> Il fait foi pour toutes les sessions de travail (interface web comme Claude Code).
> Toute décision prise en discussion doit être reportée dans le repo (ce fichier, `CLAUDE.md`, une spec ou un ADR).

---

## 1. Objectif

Évaluer la capacité de Claude à développer de façon autonome un ensemble de **micro-services en Rust** avec un **frontend React**, selon un processus rigoureux piloté par l'utilisateur.

## 2. Règles fondamentales

| Règle | Exigence |
|---|---|
| **TDD** | Les tests sont écrits avant le code, et doivent échouer avant l'implémentation (red → green → refactor). |
| **Couverture** | 100 % de couverture de tests, vérifiée et bloquante en CI. |
| **Documentation** | Tout le code produit est documenté (API publiques, modules, services). |
| **CI/CD** | GitHub Actions : chaque PR passe tous les contrôles avant merge. |
| **Pilotage** | L'utilisateur valide chaque étape avant de passer à la suivante. |

## 3. Processus par étape

### 3.1 Boucle générale

1. L'utilisateur rédige la spécification de l'étape (`specs/etape-XX-<nom>.md`).
2. Claude exécute les phases 0 à 4 de l'étape.
3. Claude présente le résultat via une Pull Request.
4. Échanges éventuels pour corriger ou améliorer.
5. L'utilisateur valide en mergeant la PR.
6. On passe à l'étape suivante.

### 3.2 Les phases d'une étape

| Phase | Contenu | Livrable | Commit |
|---|---|---|---|
| **0. Clarification** | Relecture de la spec, remontée des ambiguïtés, cas limites, hypothèses. Aucune hypothèse silencieuse. | Questions / hypothèses validées, ajoutées à la spec | `docs(spec): ...` |
| **1. Tests** | Écriture des tests à partir des critères d'acceptation. Les tests compilent mais **échouent**. | Suite de tests rouge | `test: ...` |
| **2. Code** | Implémentation minimale pour faire passer les tests, avec documentation. | Suite verte, couverture 100 % | `feat: ...` |
| **3. Revue critique** | Relecture **indépendante** (session ou sous-agent neuf, qui ne voit que la spec et le diff). | `reviews/etape-XX.md` avec sévérités (bloquant / majeur / mineur / suggestion) | `review: ...` |
| **4. Refactoring & sécurité** | Traitement point par point du rapport de revue, optimisation, validation sécurité. Tests toujours verts. | Code final, rapport de revue annoté (traité / rejeté + justification) | `refactor: ...` |

### 3.3 Conventions Git

- Une branche par étape : `etape/XX-<nom>`.
- Un commit (au minimum) par phase, au format [Conventional Commits](https://www.conventionalcommits.org/).
- Le commit de phase 1 doit faire échouer les tests : c'est la preuve du TDD.
- Le merge de la PR vaut validation de l'étape par l'utilisateur.

## 4. Qualité et tests

### 4.1 Couverture

- **Rust** : `cargo-llvm-cov`, seuil de 100 % des lignes, bloquant en CI.
- **React** : Vitest + couverture v8, seuils à 100 (lignes, fonctions, branches, statements).
- **Exclusions** : uniquement selon une politique explicite et documentée (ex. point d'entrée `main.rs` de bootstrap, code généré). Toute exclusion est justifiée dans un ADR.

### 4.2 Mutation testing

La couverture ne garantit pas que les tests vérifient quelque chose. On ajoute du mutation testing :

- **Rust** : `cargo-mutants`
- **React** : Stryker

Les mutants survivants sont traités en phase 3 / 4.

### 4.3 Types de tests

| Niveau | Outils | Portée |
|---|---|---|
| Unitaires | `cargo test`, Vitest + Testing Library | Fonctions, modules, composants |
| Intégration | `testcontainers` | Service + dépendances réelles (BDD, broker…) |
| Contrat | OpenAPI + tests de contrat | Cohérence entre services et avec le front |
| End-to-end | Playwright (à confirmer) | Parcours utilisateur complets |

### 4.4 Contrats entre services

- Une spec OpenAPI par service, générée depuis le code (ex. `utoipa`).
- Client TypeScript du front généré depuis ces specs.
- Les tests de contrat garantissent qu'un changement d'API ne casse pas les consommateurs.

## 5. Sécurité

Contrôles automatisés en CI :

- `cargo clippy -- -D warnings` (lints pedantic activés)
- `cargo fmt --check`
- `cargo-audit` (vulnérabilités connues)
- `cargo-deny` (licences, sources, CVE, doublons)
- `npm audit` / ESLint / Prettier
- CodeQL
- Dependabot
- Secret scanning GitHub

Contrôles manuels en phase 4 : validation des entrées, gestion des erreurs (pas de fuite d'information), authentification / autorisation, journalisation sans données sensibles, `unsafe` interdit sauf ADR.

## 6. Documentation

- **Rust** : `#![deny(missing_docs)]` dans chaque crate, rustdoc avec exemples testés (doctests).
- **React** : TSDoc sur les composants, hooks et fonctions exportés.
- **README par service** : rôle, API, configuration, lancement local.
- **ADR** (`docs/adr/NNNN-<titre>.md`) pour chaque décision structurante : choix techniques, exclusions de couverture, compromis.

## 7. CI/CD (GitHub Actions)

Pipeline exécuté sur chaque PR, tous les jobs bloquants :

1. Format et lint (Rust + front)
2. Build
3. Tests unitaires et d'intégration
4. Couverture (seuil 100 %)
5. Mutation testing (au minimum sur le code modifié)
6. Audits de sécurité (`cargo-audit`, `cargo-deny`, `npm audit`, CodeQL)
7. Génération de la documentation
8. Build des images Docker

Le déploiement (CD) sera défini dans une étape dédiée.

## 8. Environnement de développement

- **Poste** : Windows 11, VS Code, GitHub.
- **Recommandation** : développer dans **WSL2** ou un **devcontainer VS Code** pour être sur la même plateforme que la CI (Ubuntu).
- **Fins de ligne** : `.gitattributes` imposant LF.
- **Local** : `docker-compose` pour lancer l'ensemble des services et leurs dépendances.

## 9. Format des spécifications

Une spec par étape, fichier `specs/etape-XX-<nom>.md`, avec la structure suivante :

```markdown
# Étape XX — <Nom>

## 1. Contexte et objectif
Pourquoi cette étape, ce qu'elle apporte.

## 2. User stories
- En tant que <rôle>, je veux <action> afin de <bénéfice>.

## 3. Critères d'acceptation
### CA-01 — <titre>
- **Given** <contexte>
- **When** <action>
- **Then** <résultat attendu>

## 4. Contrat d'API
Endpoints, méthodes, payloads (requête / réponse), codes d'erreur.
(ou référence à un fichier OpenAPI)

## 5. Règles métier et invariants

## 6. Exigences non fonctionnelles
Performance, sécurité, logs, observabilité.

## 7. Hors périmètre
Ce que cette étape ne traite pas.

## 8. Questions / hypothèses (complété en phase 0)
```

Chaque critère d'acceptation (CA-XX) est référencé dans le nom ou la documentation des tests correspondants : traçabilité spec → test.

## 10. Répartition des outils

| Usage | Outil |
|---|---|
| Rédiger et challenger les specs, discuter d'architecture, relire les rapports de revue | Interface web Claude |
| Exécuter les phases 0 à 4 : code, tests, couverture, git, corrections de CI | **Claude Code dans VS Code** |
| Revue critique indépendante (phase 3) | Sous-agent ou nouvelle session Claude Code |

### Mise en place côté Claude Code

- **`CLAUDE.md`** à la racine : règles du projet (synthèse de ce document), relu à chaque session.
- **Commandes de phase** dans `.claude/commands/` : `/phase-0-clarif`, `/phase-1-tests`, `/phase-2-code`, `/phase-3-review`, `/phase-4-refactor`.
- **Plan mode** pour valider l'approche avant écriture quand l'étape est complexe.

> **Règle d'or** : Claude Code ne voit pas les discussions de l'interface web. Le repo est la seule source de vérité.

## 11. Arborescence cible (indicative)

```
.
├── CLAUDE.md
├── Instructions.md
├── .gitattributes
├── .github/
│   ├── workflows/
│   └── dependabot.yml
├── .claude/
│   └── commands/
├── specs/
├── reviews/
├── docs/
│   └── adr/
├── services/            # workspace Cargo, un crate par micro-service
│   └── <service>/
├── crates/              # bibliothèques partagées
├── frontend/            # React + Vite + Vitest
├── Cargo.toml           # workspace
├── deny.toml
└── docker-compose.yml
```

## 12. Stack technique

Chaque choix ci-dessous fait l'objet d'un ADR initial à l'étape 0.

### 12.1 Backend (Rust)

| Domaine | Choix | Justification |
|---|---|---|
| Runtime async | **Tokio** | Standard de fait de l'écosystème async Rust. |
| Framework web | **axum** | Maintenu par l'équipe Tokio, middlewares `tower` (timeout, CORS, tracing, rate limiting), handlers testables sans serveur. |
| Style d'API | **REST / JSON** | Simple, outillé, documentable en OpenAPI. |
| Documentation d'API | **utoipa** | OpenAPI générée depuis le code, base du client TypeScript. |
| Sérialisation | **serde** | Standard. |
| Erreurs | **thiserror** + réponses **RFC 9457** (Problem Details) | Erreurs typées côté domaine, format d'erreur HTTP standard sans fuite d'information. |
| Validation des entrées | **validator** | Validation déclarative des payloads. |
| Logs / observabilité | **tracing** + identifiant de corrélation par requête | Logs structurés dès le départ, prêts pour la communication inter-services. |
| Configuration | Variables d'environnement | Conforme 12-factor, compatible Docker. |

### 12.2 Données

- **PostgreSQL par défaut**, accès via **sqlx** : requêtes vérifiées à la compilation, migrations intégrées, mode offline (`.sqlx` versionné) pour compiler en CI sans base.
- Le type `JSONB` couvre les besoins de données semi-structurées.
- **MongoDB** uniquement sur besoin justifié par ADR, service par service.
- **Une base par service** : aucun service n'accède à la base d'un autre.

### 12.3 Architecture interne d'un service

Architecture **hexagonale légère** :

- **Domaine** : logique métier pure, sans dépendance d'infrastructure, testée unitairement.
- **Ports** : traits (ex. repositories) définissant les dépendances du domaine.
- **Adaptateurs** : implémentations HTTP (axum) et persistance (sqlx).

Stratégie de test associée :

- domaine et handlers testés avec des mocks des ports (`mockall`) ;
- adaptateurs de persistance testés contre une vraie base via `testcontainers`.

### 12.4 Communication et exposition

- **Pas de communication inter-services** dans un premier temps. Elle sera introduite par une étape et un ADR dédiés.
- **Reverse proxy** (Traefik ou nginx, à trancher en ADR) comme point d'entrée unique du frontend vers les services.
- **Authentification** : si nécessaire, JWT validé par chaque service, logique mutualisée dans un crate partagé (`crates/`).

### 12.5 Frontend

- **React + TypeScript**, build **Vite**.
- Tests : **Vitest** + **Testing Library**, couverture v8 à 100 %, mutation testing **Stryker**.
- Client d'API généré depuis les specs OpenAPI des services.

## 13. Prochaine étape

**Étape 0 — Bootstrap** :

- workspace Cargo et application React ;
- workflow GitHub Actions avec tous les contrôles ;
- `CLAUDE.md`, template de spec, commandes de phase ;
- ADR initiaux reprenant les choix de la section 12.
