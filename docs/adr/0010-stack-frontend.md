# ADR 0010 — Stack frontend React / TypeScript / Vite

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

Le frontend doit respecter les mêmes exigences de qualité que le backend.

## Décision

**React + TypeScript** (mode strict), build **Vite**. Tests **Vitest** + **Testing Library** (jsdom), couverture **v8** à 100 % (lignes, fonctions, branches, statements). Mutation testing **Stryker** (runner Vitest). Qualité : **ESLint** (typescript-eslint, react-hooks) et **Prettier**. Documentation **TSDoc** sur les exports. Image Docker : build multi-étapes, servie par nginx non root.

## Conséquences

Outillage homogène (Vite et Vitest partagent la configuration). Le client d'API sera généré depuis OpenAPI (ADR 0003).

Épinglages imposés par la compatibilité des outils au moment du bootstrap (à lever dès que possible) :

- **TypeScript 6.0** au lieu de 7 : `typescript-eslint` exige `typescript < 6.1`.
- **Vitest 4.1** au lieu de 5 : avec Vitest 5, le runner Stryker 10 n'exécute aucun test pendant les runs de mutation (tous les mutants « survivent »).
- Le template Vite propose désormais oxlint ; il est remplacé par **ESLint**, conformément aux instructions du projet.
