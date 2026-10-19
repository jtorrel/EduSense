# Étape 00 — Bootstrap

> Spec rédigée par Claude à partir de la section 13 de `Instructions.md`, à valider par
> l'utilisateur lors de la revue de la PR. Les hypothèses prises sont listées en section 8.

## 1. Contexte et objectif

Mettre en place le socle du projet avant toute fonctionnalité métier : workspace Cargo,
application React, pipeline CI complet, règles pour Claude Code (`CLAUDE.md`, commandes de
phase), template de spec et ADR initiaux.

Pour que chaque contrôle de la CI (build, tests, couverture, mutation, audits, docs, images
Docker) porte sur du code réel, l'étape livre un **squelette exécutable** (*walking skeleton*) :
un service Rust minimal `health` et un frontend minimal.

## 2. User stories

- En tant qu'utilisateur pilote, je veux que chaque PR soit bloquée si un contrôle qualité
  échoue, afin de ne jamais merger du code non conforme aux règles du projet.
- En tant que Claude Code, je veux un `CLAUDE.md` et des commandes de phase, afin d'appliquer
  le même processus à chaque session sans dépendre des discussions de l'interface web.
- En tant que développeur, je veux un service et un front d'exemple, afin de disposer d'un
  modèle (architecture, tests, documentation) à reproduire pour les futurs services.

## 3. Critères d'acceptation

### CA-01 — Endpoint de santé
- **Given** le service `health` démarré
- **When** un client appelle `GET /health`
- **Then** il reçoit `200 OK` avec le corps JSON `{"status":"ok"}`

### CA-02 — Spécification OpenAPI exposée
- **Given** le service `health` démarré
- **When** un client appelle `GET /api-docs/openapi.json`
- **Then** il reçoit `200 OK` avec un document OpenAPI 3.1 décrivant au moins `GET /health`

### CA-03 — Erreurs au format RFC 9457
- **Given** le service `health` démarré
- **When** un client appelle une route inexistante
- **Then** il reçoit `404 Not Found`, `Content-Type: application/problem+json`, et un corps
  contenant `type`, `title`, `status` sans détail interne
- **And** une méthode non supportée sur une route existante (ex. `POST /health`) renvoie
  `405 Method Not Allowed` au même format, avec l'en-tête `Allow`

### CA-04 — Identifiant de corrélation
- **Given** le service `health` démarré
- **When** un client envoie une requête avec l'en-tête `x-request-id`
- **Then** la réponse contient le même `x-request-id`
- **And** sans en-tête fourni, la réponse contient un `x-request-id` généré (UUID)
- **And** un `x-request-id` invalide (vide, plus de 128 caractères, ou caractère hors
  `[A-Za-z0-9._-]`) est ignoré et remplacé par un UUID généré

### CA-05 — Configuration par variables d'environnement
- **Given** la variable `HEALTH_BIND_ADDR` absente
- **When** le service charge sa configuration
- **Then** l'adresse d'écoute est `0.0.0.0:8080`
- **And** si la variable contient une adresse valide, elle est utilisée
- **And** si elle contient une valeur invalide, le chargement échoue avec une erreur explicite

### CA-06 — Frontend minimal
- **Given** l'application React
- **When** la page d'accueil est rendue
- **Then** elle affiche le titre de niveau 1 « EduSense »

### CA-07 — Pipeline CI
- **Given** une PR ouverte sur `main`
- **When** la CI s'exécute
- **Then** les jobs suivants s'exécutent et sont tous bloquants : format/lint (Rust + front),
  build, tests, couverture 100 %, mutation testing, audits de sécurité (`cargo-audit`,
  `cargo-deny`, `npm audit`, CodeQL), documentation, build des images Docker

## 4. Contrat d'API

Service `health` :

| Méthode | Chemin | Réponse |
|---|---|---|
| `GET` | `/health` | `200` — `{"status":"ok"}` |
| `GET` | `/api-docs/openapi.json` | `200` — document OpenAPI (JSON) |
| `*` | route existante, méthode non supportée | `405` — `application/problem+json`, en-tête `Allow` |
| `*` | toute autre route | `404` — `application/problem+json` |

## 5. Règles métier et invariants

Aucune règle métier à cette étape.

## 6. Exigences non fonctionnelles

- Logs structurés via `tracing`, chaque requête tracée avec son `x-request-id`.
- Aucune information interne (trace, chemin, version) dans les réponses d'erreur.
- Image Docker du service exécutée en utilisateur non root.

## 7. Hors périmètre

- Base de données (aucun service n'en a encore besoin ; `sqlx` sera introduit avec le premier
  service persistant).
- Génération du client TypeScript depuis OpenAPI et tests de contrat (première étape avec un
  front consommant une API).
- Tests end-to-end (Playwright à confirmer).
- Déploiement (CD).
- Communication inter-services, authentification.

## 8. Questions / hypothèses (complété en phase 0)

| # | Question | Hypothèse retenue |
|---|---|---|
| H1 | La section 13 ne fournit pas de spec détaillée. | Claude rédige cette spec ; elle est validée avec la PR. |
| H2 | Comment exercer couverture, mutation et build Docker sans code métier ? | Squelette `services/health` + front minimal (ADR 0014). Le service pourra être supprimé ou conservé comme modèle. |
| H3 | Reverse proxy Traefik ou nginx ? | ADR 0013 au statut **Proposé** (nginx recommandé). À trancher par l'utilisateur ; non mis en place à cette étape au-delà du front servi par nginx. |
| H4 | Mutation testing « au minimum sur le code modifié ». | Rust : `cargo mutants --in-diff` sur les PR. Front : Stryker sur tout le code (volume faible). |
| H5 | Seuil de score de mutation ? | 100 % des mutants tués (ou justifiés). Revu si trop coûteux. |
| H6 | Version de Rust. | Toolchain stable épinglée dans `rust-toolchain.toml`, édition 2024. |
| H7 | Environnement local Windows sans Docker ni WSL. | Le devcontainer est fourni ; les contrôles Docker et `cargo-audit` sont validés par la CI. |
| H8 | Crate partagé pour Problem Details / corrélation ? | Pas tant qu'il n'y a qu'un service (YAGNI) ; extraction dans `crates/` à l'arrivée du 2ᵉ service. |
| H9 | Réponse à une méthode non supportée (revue R-01) ? | `405` Problem Details avec en-tête `Allow`, ajouté à CA-03 et documenté dans l'OpenAPI. |
| H10 | Faut-il valider le `x-request-id` fourni (revue R-05) ? | Oui : 1 à 128 caractères `[A-Za-z0-9._-]`, sinon UUID généré (CA-04). Évite la pollution des logs. |

### Écarts au processus TDD (revue R-07)

- `env_filter_uses_valid_directives` : l'attendu du test de phase 1 a été corrigé en phase 2.
  `EnvFilter` réordonne les directives (de la plus spécifique à la plus générale) ; le test
  supposait l'ordre de saisie. Le comportement testé est inchangé.
- `tests/request_logging.rs` : ajouté en phase 2, rouge avant le correctif (les requêtes
  n'étaient journalisées qu'en `DEBUG`), pour couvrir l'exigence §6 que la phase 1 avait omise.
- Phase 4 : les tests des constats R-01, R-03, R-05 et R-12 ont été écrits et constatés rouges
  avant les corrections.
