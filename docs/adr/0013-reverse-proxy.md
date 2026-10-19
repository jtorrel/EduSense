# ADR 0013 — Reverse proxy : nginx

- **Statut** : Proposé — à trancher par l'utilisateur
- **Date** : 2026-10-19

## Contexte

Le frontend doit accéder aux services via un point d'entrée unique (section 12.4 des instructions). Deux candidats : Traefik et nginx.

## Décision

Proposition : **nginx**.

- Configuration statique simple et lisible, déjà utilisée pour servir le front (une seule technologie à maîtriser).
- Très répandu, image légère, exécutable en non-root (`nginxinc/nginx-unprivileged`).
- Traefik apporte la découverte dynamique (labels Docker) et Let's Encrypt intégré, utiles surtout avec beaucoup de services ou un déploiement dynamique, hors périmètre actuel.

Le routage `/api/<service>` sera mis en place avec la première étape où le front consomme une API.

## Conséquences

Si Traefik est retenu, cet ADR sera remplacé ; impact limité car aucun routage n'est encore implémenté.
