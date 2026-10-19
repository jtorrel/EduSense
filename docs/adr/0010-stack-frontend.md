# ADR 0010 — Stack frontend React / TypeScript / Vite

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

Le frontend doit respecter les mêmes exigences de qualité que le backend.

## Décision

**React + TypeScript** (mode strict), build **Vite**. Tests **Vitest** + **Testing Library** (jsdom), couverture **v8** à 100 % (lignes, fonctions, branches, statements). Mutation testing **Stryker** (runner Vitest). Qualité : **ESLint** (typescript-eslint, react-hooks) et **Prettier**. Documentation **TSDoc** sur les exports. Image Docker : build multi-étapes, servie par nginx non root.

## Conséquences

Outillage homogène (Vite et Vitest partagent la configuration). Le client d'API sera généré depuis OpenAPI (ADR 0003).
