# Avis étudiants — EduSense

Micro-service Rust pour recueillir les avis des étudiants de la Normandie Web School sous forme de tickets.
Spécification : `avis-etudiants/spec-v1.md` dans le projet EduSense.

- **Étudiant** : ouvre un ticket (date, catégorie, intervenant, note 0–5, commentaire), suit son statut, lit la réponse, peut annuler un ticket non traité.
- **Admin** : voit tous les tickets, répond (une réponse par ticket), change le statut.
- **Superadmin** : en plus, supprime des tickets et gère la liste des admins.

Stack : axum · askama (pages générées côté serveur) · sqlx (MySQL) · connexion Google OAuth.

## 1. Préparer Google (une seule fois)

1. Ouvrir [Google Cloud Console](https://console.cloud.google.com/) avec un compte de l'école, puis créer (ou choisir) un projet.
2. **API et services → Écran de consentement OAuth** : type d'utilisateur **Interne**. Seuls les comptes du Workspace pourront se connecter, et aucune validation Google n'est nécessaire.
3. **API et services → Identifiants → Créer des identifiants → ID client OAuth**
   - Type : **Application Web**
   - URI de redirection autorisés : `https://VOTRE-DOMAINE/auth/callback` (et `http://localhost:8080/auth/callback` pour les tests)
4. Recopier l'ID client et le code secret dans `GOOGLE_CLIENT_ID` et `GOOGLE_CLIENT_SECRET`.

Le service vérifie lui-même, côté serveur, que l'adresse est vérifiée par Google et appartient au domaine `ALLOWED_DOMAIN`.

## 2. Préparer la base

Une base **dédiée**, distincte de la base ERP `auto` :

```sql
CREATE DATABASE edusense_avis CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER 'avis'@'%' IDENTIFIED BY 'un-mot-de-passe-solide';
GRANT ALL ON edusense_avis.* TO 'avis'@'%';
```

Les tables sont créées automatiquement au démarrage (dossier `migrations/`).

> **MySQL 5.7 et TLS** : la négociation TLS de MySQL 5.7 n'est pas compatible avec la bibliothèque TLS du service. Si la base est sur la même machine ou sur un réseau privé, ajoutez `?ssl-mode=disabled` à `DATABASE_URL`. Si la connexion passe par un réseau non sûr, utilisez un tunnel (SSH, WireGuard) ou MySQL 8.

## 3. Configurer

```bash
cp .env.example .env
openssl rand -hex 32     # → à coller dans SESSION_SECRET
```

| Variable | Rôle |
|---|---|
| `DATABASE_URL` | `mysql://avis:mdp@hote:3306/edusense_avis?ssl-mode=disabled` |
| `PUBLIC_BASE_URL` | URL publique, sans `/` final. En `https://`, les cookies sont marqués « Secure » |
| `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` | Client OAuth (étape 1) |
| `ALLOWED_DOMAIN` | `normandiewebschool.fr` |
| `SUPERADMINS` | Emails séparés par des virgules |
| `CATEGORIES` | Catégories séparées par des `;` |
| `SESSION_SECRET` | ≥ 32 caractères. Le changer déconnecte tout le monde |
| `BIND_ADDR` | `0.0.0.0:8080` par défaut |

Les **admins** se gèrent dans l'interface (menu « Admins », réservé aux superadmins). Le retrait d'un admin prend effet immédiatement.

## 4. Lancer

**En production (VPS EduSense)** : publier une release avec le tag `avis-etudiants/vX.Y.Z`, puis `edusense deploy avis vX.Y.Z` sur le VPS (voir `infra/README.md`). La base MySQL 8 du VPS est configurée automatiquement ; la remarque sur MySQL 5.7 ne s'applique pas.

**En local avec Docker** (le service et un MySQL 5.7), depuis ce dossier :

```bash
docker compose up --build        # → http://localhost:8080
```

**Image seule**, depuis la racine du monorepo :

```bash
docker build --build-arg SERVICE=avis-etudiants -t avis-etudiants .
```

**Sans Docker**, depuis ce dossier : `cargo run --release` (Rust ≥ 1.85).

`/healthz` répond `ok` pour la supervision.

## 5. Développer sans Google

Une connexion simulée existe pour tester en local. Elle n'est **compilée qu'avec la feature `dev-login`** (jamais dans l'image Docker) et doit en plus être activée par `DEV_LOGIN=1` :

```bash
DEV_LOGIN=1 cargo run --features dev-login
# puis http://localhost:8080 → champ « Connexion simulée »
```

## 6. Tests

```bash
cargo test                       # règles métier : droits, statuts, validation
./scripts/smoke-test.sh          # bout en bout, sur un service lancé en mode dev-login
```

Le test de bout en bout (42 vérifications) couvre : cloisonnement entre étudiants, accès admin et superadmin, CSRF, validation, réponse, statuts, annulation, suppression, retrait d'un admin et déconnexion. Il a été exécuté avec succès sur MySQL 5.7.44 et MariaDB 10.11.

## Organisation du code

```
src/
  main.rs          démarrage, migrations, serveur
  config.rs        variables d'environnement
  domain.rs        règles métier pures (rôles, statuts, droits, validation) + tests
  db.rs            requêtes SQL (modifications en transaction avec verrou de ligne)
  auth.rs          OAuth Google, session (cookie chiffré), extracteurs de rôle
  web.rs           état partagé, erreurs, mise en forme pour les templates
  routes/          pages étudiant et admin
templates/         HTML (askama)
migrations/        schéma SQL
```

## Sécurité

- Session dans un cookie **chiffré et signé** (HttpOnly, SameSite=Lax, Secure en HTTPS), valable 12 h. Le rôle est relu en base à chaque requête.
- Jeton **CSRF** sur tous les formulaires ; paramètre `state` vérifié au retour de Google.
- Tout le contenu affiché est échappé (askama) ; toutes les requêtes SQL sont paramétrées.
- Les droits sont vérifiés côté serveur, dans une transaction, au moment de chaque action.
- Rien n'est vraiment supprimé : les tickets « supprimés » restent en base et toutes les actions sont tracées dans `ticket_events`.
