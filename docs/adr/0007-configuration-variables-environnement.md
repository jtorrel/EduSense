# ADR 0007 — Configuration par variables d'environnement

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

Les services sont déployés en conteneurs (12-factor).

## Décision

Configuration exclusivement par **variables d'environnement**, préfixées par le nom du service (ex. `HEALTH_BIND_ADDR`). Valeurs par défaut sûres pour le développement. Le chargement est une fonction pure testable (source de variables injectée) qui échoue explicitement au démarrage en cas de valeur invalide. Aucun secret versionné : `.env` ignoré par git, `.env.example` documenté.

## Conséquences

Compatible Docker / docker-compose ; pas de fichier de configuration à gérer.
