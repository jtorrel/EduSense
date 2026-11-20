# EduSense — serveur et déploiement

> Ce dossier (`infra/`) fait partie du monorepo EduSense. C'est lui qu'on copie sur le VPS.

Un VPS, plusieurs services, une commande pour déployer.

```
GitHub ── tag avis-etudiants/v1.2.0 ──► GitHub Actions : tests + image Docker ──► ghcr.io
                                                                     │
VPS :  Caddy (HTTPS automatique) ──► avis.normandiewebschool.fr ◄────┘ edusense deploy avis v1.2.0
       MySQL 8 (une base par service)     autre.normandiewebschool.fr …
```

---

## A. Installer le serveur (une seule fois, ≈ 10 min)

> **Sous Windows 11** : toutes les commandes « sur votre poste » se tapent dans **PowerShell** (menu Démarrer → « PowerShell »). `ssh` et `scp` y sont déjà installés.

**1. Chez OVH** : Ubuntu 26.04, en collant votre clé SSH **existante**. Pour l'afficher dans PowerShell :

```powershell
dir $HOME\.ssh                          # repérer le fichier .pub (id_ed25519.pub ou id_rsa.pub)
Get-Content $HOME\.ssh\id_ed25519.pub   # copier la ligne entière
```

**2. Copier le dossier sur le serveur.** Dans PowerShell, placez-vous dans le dossier du monorepo `edusense`, puis copiez son sous-dossier `infra` sous le nom `edusense-infra` (le **`:`** après l'IP signifie « sur le serveur ») :

```powershell
cd $HOME\Documents\edusense           # adapter selon l'emplacement du monorepo
scp -r infra ubuntu@IP_DU_VPS:edusense-infra
ssh ubuntu@IP_DU_VPS
```

**3. Sur le serveur** (vous êtes maintenant connecté au VPS) :

```bash
sed -i 's/\r$//' edusense-infra/setup-vps.sh edusense-infra/edusense edusense-infra/socle/*   # fins de ligne Windows → Linux, par sécurité
sudo bash edusense-infra/setup-vps.sh
```

Le script :
- met le système à jour et active les mises à jour de sécurité automatiques ;
- n'autorise plus que la connexion par clé SSH (plus de mot de passe, plus de root) ;
- ouvre seulement les ports 22, 80 et 443 ;
- installe Docker, Caddy (HTTPS) et MySQL 8 ;
- installe la commande `edusense` ;
- programme une sauvegarde des bases chaque nuit (conservée 14 jours dans `/srv/backups`).

**4. Déconnectez-vous puis reconnectez-vous** (`exit`, puis `ssh ubuntu@IP_DU_VPS`).

**5. Autoriser le serveur à télécharger vos images GitHub** (une seule fois) :
- Sur GitHub : *Settings → Developer settings → Personal access tokens → Tokens (classic) → Generate new token*. Cochez seulement **`read:packages`** et choisissez une expiration longue.
- Sur le VPS :
  ```bash
  docker login ghcr.io -u VOTRE_PSEUDO_GITHUB
  # mot de passe : collez le jeton
  ```

---

## B. Ajouter un service (exemple : avis-etudiants)

**1. Sur GitHub** (une seule fois pour tout le monorepo) : créez le dépôt `edusense` et envoyez-y le code (le plus simple sous Windows : [GitHub Desktop](https://desktop.github.com/)).

Puis publiez une première version du service, **directement sur le site GitHub** :

*Releases* (colonne de droite du dépôt) → *Draft a new release* → *Choose a tag* : tapez **`avis-etudiants/v1.0.0`** → *Create new tag* → *Publish release*.

Le début du tag (`avis-etudiants/`) indique **quel service** publier : seul celui-là est testé, construit et publié. (Équivalent en ligne de commande : `git tag avis-etudiants/v1.0.0` puis `git push origin avis-etudiants/v1.0.0`.)

Dans l'onglet **Actions** du dépôt, attendez que tout soit vert (≈ 5 min la première fois). L'image est alors disponible sous `ghcr.io/VOTRE_COMPTE/avis-etudiants:v1.0.0`.

**2. Chez votre gestionnaire DNS** : ajoutez un enregistrement **A** `avis` qui pointe vers l'IP du VPS.

**3. Sur le VPS** :

```bash
edusense new avis avis.normandiewebschool.fr ghcr.io/VOTRE_COMPTE/avis-etudiants
nano /srv/avis/app.env
```

La base de données, son mot de passe, l'adresse publique et la clé de session sont **déjà remplis**. Ajoutez à la fin :

```
GOOGLE_CLIENT_ID=…
GOOGLE_CLIENT_SECRET=…
SUPERADMINS=jc.torrell@normandiewebschool.fr
```

(`Ctrl+O`, `Entrée` pour enregistrer, `Ctrl+X` pour quitter.) Puis :

```bash
edusense deploy avis v1.0.0
```

C'est en ligne sur `https://avis.normandiewebschool.fr`. Le certificat HTTPS est obtenu automatiquement (cela peut prendre une minute la première fois, le temps que le DNS se propage).

---

## C. Au quotidien

| Je veux… | Commande |
|---|---|
| Publier une nouvelle version | sur GitHub : *Releases → Draft a new release*, tag `avis-etudiants/v1.1.0` (attendre le vert dans Actions) |
| La mettre en ligne | sur le VPS : `edusense deploy avis v1.1.0` |
| Revenir à la version d'avant | `edusense deploy avis v1.0.0` |
| Voir ce qui tourne | `edusense status` |
| Voir les versions déployées | `edusense history avis` |
| Lire les journaux | `edusense logs avis` (`Ctrl+C` pour quitter) |
| Changer la configuration | `nano /srv/avis/app.env` puis `edusense restart avis` |
| Faire une sauvegarde maintenant | `edusense backup` |

**Filet de sécurité** : si une nouvelle version ne démarre pas correctement, `edusense deploy` le détecte, affiche les journaux et **revient tout seul à la version précédente**.

**Numéros de version** : `vMAJEUR.MINEUR.CORRECTIF`, par exemple `v1.0.1` pour un correctif, `v1.1.0` pour une nouveauté, `v2.0.0` pour un gros changement.

> Le retour arrière remet l'ancien code, mais pas l'ancienne structure de base de données. Les changements de base seront donc toujours écrits pour rester compatibles avec la version précédente.

---

## D. Bon à savoir

- **Organisation sur le VPS** :
  ```
  /srv/socle/      Caddy + MySQL (sites/ : un fichier par service)
  /srv/avis/       app.env (configuration), .env (version), history
  /srv/backups/    sauvegardes MySQL quotidiennes
  ```
- **Mettre à jour l'outil** après une modification de ce dossier : sur le VPS `rm -rf edusense-infra` (sinon `scp` copierait *dans* l'ancien dossier) ; depuis le monorepo `scp -r infra ubuntu@IP_DU_VPS:edusense-infra` ; puis sur le VPS `sudo bash edusense-infra/setup-vps.sh` (sans risque).
- **Mettre à jour Caddy et MySQL** : `cd /srv/socle && docker compose pull && docker compose up -d`.
- **Sauvegardes hors du serveur** : activez en plus les *snapshots* ou le *backup automatisé* OVH, ou copiez régulièrement `/srv/backups` ailleurs.
- **Restaurer une sauvegarde** :
  ```bash
  gunzip -c /srv/backups/mysql-AAAAMMJJ-HHMM.sql.gz | \
    docker compose -f /srv/socle/docker-compose.yml exec -T mysql sh -c 'mysql -uroot -p"$MYSQL_ROOT_PASSWORD"'
  ```
- **Mot de passe root MySQL** : dans `/srv/socle/.env`. Chaque service a son propre utilisateur, limité à sa base.
