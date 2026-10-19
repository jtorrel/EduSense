# ADR 0003 — API REST/JSON documentée par utoipa

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

Le front et les futurs services doivent partager des contrats d'API fiables (section 4.4 des instructions).

## Décision

Style **REST / JSON** sérialisé avec **serde**. Chaque service génère sa spec **OpenAPI 3.1** depuis le code avec **utoipa** et l'expose sur `GET /api-docs/openapi.json`. Le client TypeScript du front sera généré depuis ces specs (étape ultérieure), et des tests de contrat vérifieront la compatibilité.

## Conséquences

La documentation d'API ne peut pas dériver du code. Annotations utoipa à maintenir sur chaque handler et DTO.
