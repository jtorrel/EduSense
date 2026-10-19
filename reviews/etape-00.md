# Revue — Étape 00 Bootstrap

> Revue indépendante (phase 3) réalisée par un sous-agent neuf, qui n'a reçu que la spec, le diff
> de l'étape et la consigne de `/phase-3-review`. La colonne « Statut » est remplie en phase 4.

## Synthèse

L'étape est globalement conforme à la spec. CA-01 à CA-06 sont couverts par des tests dont les assertions sont réelles : corps JSON comparés exactement, en-têtes vérifiés, cas invalides de CA-05 énumérés, logs capturés sans la query string. Le code est simple et documenté (rustdoc + doctests, README du service, ADR à jour). Le workflow CI comporte bien tous les jobs exigés par CA-07, regroupés derrière un job de synthèse `CI OK`. Aucun constat bloquant.

Le constat principal est fonctionnel : une méthode non autorisée sur une route existante renvoie un `405` sans corps, pas au format RFC 9457. Ce comportement n'est pas prévu par la spec et un test le fige (R-01). Les autres constats portent sur le durcissement (en-têtes nginx, `x-request-id`), la cohérence de `ProblemDetails`, le message d'erreur de démarrage et la traçabilité/TDD.

**Contrôles exécutés en local (Windows, Git Bash) :**

| Contrôle | Résultat |
|---|---|
| `cargo fmt --all --check` | OK |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | OK, 0 warning |
| `cargo test --workspace --locked` | OK : 13 unitaires, 7 + 1 + 1 intégration, 7 doctests |
| `cargo cov` | 100 % lignes, régions et fonctions (`main.rs` bien exclu, y compris avec les chemins Windows) |
| `cargo doc --workspace --no-deps` avec `RUSTDOCFLAGS=-D warnings` | OK |
| `cargo deny check` | `advisories ok, bans ok, licenses ok, sources ok` ; un seul avertissement non bloquant (doublon `syn`) |
| `cargo mutants --workspace` | 18 mutants : **4 tués, 0 survivant**, 0 timeout, 14 non viables (`Default::default()` / `Json::new()` sur des types sans `Default`) |
| Front : `npm run lint`, `format:check`, `typecheck` | OK |
| Front : `npm run test:coverage` | 100 % sur toutes les métriques |
| Front : `npm audit` | 0 vulnérabilité |
| Front : `npm run mutation` (Stryker) | 1 mutant, **1 tué, score 100 %** |

**Non exécutés localement** (la CI fait foi) : `cargo audit` (non installé), builds Docker, CodeQL.

**Vérifications ad hoc**, faites sur une copie du dépôt en dehors du repo puis supprimée :
- `POST /health` → `405`, `content-length: 0`, pas de `Content-Type` problem+json.
- `x-request-id` de 8000 caractères → renvoyé tel quel.
- `HEALTH_BIND_ADDR=localhost` → le binaire affiche `Error: InvalidBindAddr { value: "localhost" }`.

## Constats

| ID | Sévérité | Fichier:ligne | Constat | Proposition | Statut |
|---|---|---|---|---|---|
| R-01 | majeur | services/health/src/adapters/http/mod.rs:59-62 ; services/health/tests/http_api.rs:36-41 | Une méthode non autorisée sur une route existante (`POST /health`, `DELETE /api-docs/openapi.json`) renvoie le `405` par défaut d'axum : corps vide, pas de `application/problem+json`. Cela contredit l'ADR 0004 (toute erreur HTTP au format RFC 9457) et le contrat §4 de la spec (« `*` toute autre route → 404 problem+json »). Le test `ca01_health_rejects_other_methods` fige ce `405` alors que la spec ne le prévoit pas : c'est une hypothèse silencieuse, contraire à CLAUDE.md. | Ajouter `.method_not_allowed_fallback(problem::method_not_allowed)`, qui renvoie `ProblemDetails::from_status(StatusCode::METHOD_NOT_ALLOWED)` (en conservant l'en-tête `Allow` si possible). Consigner le comportement 405 dans la spec (§4 et §8) et compléter le test : content-type problem+json et corps attendu. Le test relève de CA-03 plutôt que de CA-01. | **Traité** — `method_not_allowed_fallback` → `405` Problem Details, `Allow` conservé ; test `ca03_unsupported_method_returns_problem_details` ; spec §3/§4/§8 (H9). |
| R-02 | mineur | frontend/nginx.conf:12-15 | Le `add_header Cache-Control` du bloc `location /assets/` désactive l'héritage des `add_header` du niveau `server` (règle nginx). Les JS/CSS servis depuis `/assets/` n'ont donc plus `X-Content-Type-Options`, `Content-Security-Policy`, `X-Frame-Options` ni `Referrer-Policy`. En plus, `expires 1y` émet déjà un `Cache-Control: max-age`, d'où un second en-tête `Cache-Control`. | Ajouter `add_header_inherit merge;` (nginx ≥ 1.29.3) ou répéter les en-têtes de sécurité dans `location /assets/` (snippet `include`). Garder un seul mécanisme de cache, par exemple `add_header Cache-Control "public, max-age=31536000, immutable" always;` sans `expires`. | **Traité** — en-têtes dans `frontend/security-headers.conf`, inclus au niveau `server` et dans `location /assets/` ; `expires` retiré (un seul `Cache-Control`). Non vérifié localement (pas de Docker). |
| R-03 | mineur | services/health/src/adapters/http/problem.rs:17-28, 57 ; test 116-129 | Les champs de `ProblemDetails` sont publics et `status` est un `u16` libre. Un statut invalide (ex. 1000) donne une réponse HTTP 500 dont le corps annonce toujours `"status": 1000`, ce qui viole la RFC 9457 (le membre `status` doit correspondre au code HTTP). Le test `ca03_into_response_with_invalid_status_falls_back_to_500` entérine cette incohérence : il ne vérifie que le statut HTTP. | Rendre l'état invalide non représentable : stocker un `StatusCode` sérialisé via `serialize_with` en `u16`, ou rendre les champs privés avec des constructeurs. À défaut, réécrire `status` et `title` dans le corps lors du repli 500, et vérifier le corps dans le test. | **Traité** — `status: StatusCode` sérialisé en entier (`serialize_with`, schéma `u16`) : un statut invalide n'est plus représentable ; test de repli 500 supprimé, sans objet. |
| R-04 | mineur | services/health/src/main.rs:12-16 | Si la configuration est invalide, `main` renvoie `Box<dyn Error>`, que Rust affiche en `Debug` (`Error: InvalidBindAddr { value: "localhost" }`). Le message explicite exigé par CA-05 (`HEALTH_BIND_ADDR invalide : « … » n'est pas une adresse ip:port`, implémenté dans `Display`) n'est jamais montré, et l'erreur n'est pas journalisée en JSON alors que le subscriber est déjà installé. | Dans `main`, journaliser l'erreur au format `Display` (`tracing::error!(error = %e, "configuration invalide")`) puis sortir avec `std::process::exit(1)`. Variante : un type d'erreur de `main` dont `Debug` délègue à `Display`. | **Traité** — `main.rs` journalise `error!(%error, "configuration invalide")` (Display) puis `exit(1)` ; vérifié manuellement. |
| R-05 | mineur | services/health/src/adapters/http/mod.rs:69, 74-86 | Le `x-request-id` fourni par le client est repris, renvoyé et journalisé sans aucune validation : longueur testée jusqu'à 8000 caractères, jeu de caractères quelconque. Le JSON échappe la valeur, donc pas d'injection de logs, mais le client peut gonfler les logs et usurper ou polluer la corrélation (Instructions §5, validation des entrées). | Accepter l'identifiant seulement s'il est court et sûr (ex. ≤ 128 caractères, `[A-Za-z0-9._-]`), sinon en générer un, via un `MakeRequestId` personnalisé ou un middleware placé avant `SetRequestIdLayer`. Ajouter un test CA-04 « identifiant invalide → UUID généré » et consigner la règle dans la spec. | **Traité** — middleware `discard_invalid_request_id` (1 à 128 caractères `[A-Za-z0-9._-]`, sinon UUID généré) ; tests `ca04_accepts_request_id_at_max_length` et `ca04_replaces_invalid_request_id` ; spec CA-04/H10, ADR 0006. |
| R-06 | mineur | services/health/src/telemetry.rs:52-78 | Les tests `env_filter_defaults_to_info`, `env_filter_uses_valid_directives`, `env_filter_falls_back_to_default_on_invalid_directives` et `subscriber_applies_filter` ne référencent aucun critère ni exigence. CLAUDE.md impose la traçabilité de chaque test. | Les relier à l'exigence §6 (docstring « Exigence §6 — … », comme dans `request_logging.rs`). Mieux : ajouter dans la spec un critère sur les logs (ex. CA-08) et préfixer les tests en conséquence. | **Traité** — docstrings « Exigence §6 — … » sur les quatre tests de `telemetry.rs`. |
| R-07 | mineur | services/health/tests/request_logging.rs (ajouté au commit `feat`) ; services/health/src/telemetry.rs:59-63 | Deux écarts au processus TDD : le test `request_logging` a été écrit en phase 2 et non dans le commit `test:` de phase 1, et l'attendu de `env_filter_uses_valid_directives` a été modifié en phase 2. Les deux écarts sont expliqués dans le message du commit `feat`, mais pas dans le dépôt, alors que CLAUDE.md demande de consigner toute hypothèse dans la spec. | Consigner ces deux écarts dans la section 8 de la spec (ou dans ce rapport, annoté en phase 4), afin que la trace TDD reste lisible sans l'historique git. | **Traité** — écarts consignés dans la spec, §8 « Écarts au processus TDD ». |
| R-08 | mineur | README.md:54 ; .github/workflows/codeql.yml:15-34 | CA-07 exige que CodeQL soit bloquant. Or le workflow CodeQL est hors de `CI OK`, et un job d'analyse CodeQL réussit même s'il trouve des alertes. Rendre « le check CodeQL » obligatoire ne bloque donc pas une PR qui introduit une vulnérabilité. | Dans le README, préciser la configuration à faire : règle de ruleset « Require code scanning results » (CodeQL, seuil d'alerte ex. *High or higher* / erreurs), en plus du check requis `CI OK`. | **Traité** — README : ruleset avec check `CI OK` et règle « Require code scanning results » (CodeQL). |
| R-09 | mineur | frontend/eslint.config.js:31-33 | CLAUDE.md impose « TSDoc sur tout export du front », mais `tsdoc/syntax` ne contrôle que la syntaxe des commentaires existants, pas leur présence. Contrairement à `missing_docs` côté Rust, un export non documenté passe la CI. | Ajouter `eslint-plugin-jsdoc` avec `jsdoc/require-jsdoc` (`publicOnly: true`, composants et fonctions exportés), en gardant `tsdoc/syntax` pour la syntaxe. | **Traité** — `eslint-plugin-jsdoc`, règle `jsdoc/require-jsdoc` (`publicOnly`) ; vérifié : la suppression de la TSDoc de `App` fait échouer le lint. |
| R-10 | suggestion | .github/workflows/ci.yml (toutes les étapes `uses:`), ci.yml:76 | Les actions tierces sont référencées par tag mutable (`@v2`, `@v3`…), et `${{ github.base_ref }}` est interpolé directement dans un script shell. Le risque est faible (`base_ref` est contrôlé par les mainteneurs), mais ce n'est pas la pratique recommandée. Aucun job n'a de `timeout-minutes`. | Épingler les actions par SHA (Dependabot `github-actions` est déjà configuré pour les mettre à jour). Passer `base_ref` par `env:` (`BASE_REF: ${{ github.base_ref }}` puis `"origin/$BASE_REF...HEAD"`). Ajouter un `timeout-minutes` par job. | **Traité** — actions épinglées par SHA (résolus avec `git ls-remote`, tag en commentaire), `BASE_REF` passé par `env`, `timeout-minutes` sur chaque job (60 pour les mutants). |
| R-11 | suggestion | services/health/Dockerfile:6-8 | Aucun cache des dépendances : toute modification de source recompile toutes les dépendances. De plus, la copie de `rust-toolchain.toml` (composants `rustfmt`, `clippy`) peut déclencher un téléchargement de ces composants, inutiles pour le build. | Utiliser `cargo-chef` ou des cache mounts BuildKit (`RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target …`, en copiant le binaire hors du cache). Ne pas copier `rust-toolchain.toml` dans l'image, ou le surcharger avec `RUSTUP_TOOLCHAIN=1.97.1`. | **Traité** — caches BuildKit (registre + `target`), binaire copié hors cache, `rust-toolchain.toml` non copié et `RUSTUP_TOOLCHAIN=1.97.1`. Non vérifié localement (pas de Docker). |
| R-12 | suggestion | services/health/src/adapters/http/mod.rs:39-44, 94-99 | Le schéma `ProblemDetails` est déclaré dans les composants OpenAPI, mais aucune réponse d'erreur ne le référence. Le contrat généré ne documente donc pas les réponses 404/405 problem+json, qui serviront de base au futur client TypeScript. | Documenter les réponses d'erreur (404 et 405 après R-01) avec `content_type = "application/problem+json"` et `body = ProblemDetails`, par exemple via une réponse réutilisable `IntoResponses`. Étendre le test CA-02 en conséquence. | **Traité** — réponse `405` `application/problem+json` → `ProblemDetails` documentée sur `GET /health` ; test CA-02 étendu. Le `404` de repli n'est rattaché à aucun chemin : il n'est pas décrit par l'OpenAPI (limite d'OpenAPI, pas de chemin « joker »). |

## Phase 4 — Traitement

12 constats traités, aucun rejeté. Les tests liés à R-01, R-03, R-05 et R-12 ont été écrits
avant les corrections et constatés rouges.

### Contrôles après corrections

| Contrôle | Résultat |
|---|---|
| `cargo fmt --check`, `cargo clippy -D warnings` (pedantic) | OK |
| `cargo test --workspace` | OK : 12 unitaires, 9 + 1 + 1 d'intégration, 7 doctests |
| `cargo cov` | 100 % (lignes, régions, fonctions) |
| `cargo mutants --workspace` | 29 mutants : 11 tués, 0 survivant, 18 non viables |
| `cargo deny check`, `cargo doc -D warnings` | OK |
| Front : format, lint (avec `jsdoc/require-jsdoc`), types, build | OK |
| Front : couverture / Stryker / `npm audit` | 100 % / 100 % / 0 vulnérabilité |
| Docker, CodeQL, `cargo audit` | non exécutables localement : validés par la CI de la PR |

### Validation sécurité manuelle

| Point | Conclusion |
|---|---|
| Validation des entrées | Seules entrées : `HEALTH_BIND_ADDR` (parsée en `SocketAddr`, échec explicite au démarrage), `RUST_LOG` (repli sur `info` si invalide) et l'en-tête `x-request-id` (validé, R-05). Aucun corps de requête accepté. |
| Erreurs sans fuite d'information | `404` / `405` au format RFC 9457 générique (`about:blank` + raison standard) ; aucune trace, version ni chemin interne. `server_tokens off` côté nginx. |
| Authentification / autorisation | Sans objet : endpoints publics de supervision, aucune donnée. |
| Journalisation | JSON ; le span contient méthode, chemin (sans query string, testé) et identifiant de corrélation validé. Aucune donnée sensible. |
| `unsafe` | Interdit par `unsafe_code = "forbid"` au niveau du workspace ; aucune occurrence. |
| Conteneurs | Images d'exécution non root (`distroless:nonroot`, `nginx-unprivileged`), sans shell pour le service. |
| Chaîne d'approvisionnement | `cargo-deny` (sources crates.io uniquement, licences, avis RustSec), `npm audit` (override `qs`), actions GitHub épinglées par SHA, Dependabot. |
