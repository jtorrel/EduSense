# ADR 0014 — Service squelette health et frontend minimal

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

À l'étape 0, sans code métier, les contrôles de CI (couverture, mutation, Docker, docs) n'auraient rien à vérifier.

## Décision

Livrer un *walking skeleton* : un service `services/health` (`GET /health`, OpenAPI, Problem Details, `x-request-id`, configuration par environnement) et un frontend minimal. Le service sert de **modèle** pour les futurs services (structure, tests, documentation, Dockerfile).

## Conséquences

Chaîne complète validée dès l'étape 0. Le service pourra être supprimé ou conservé (sonde de supervision) lors d'une étape ultérieure.
