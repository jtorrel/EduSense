# Frontend EduSense

Application **React + TypeScript** construite avec **Vite** ([ADR 0010](../docs/adr/0010-stack-frontend.md)).

## Scripts

| Commande                          | Rôle                                                      |
| --------------------------------- | --------------------------------------------------------- |
| `npm run dev`                     | Serveur de développement                                  |
| `npm run build`                   | Vérification des types puis build de production (`dist/`) |
| `npm run lint`                    | ESLint (typescript-eslint strict, react-hooks, TSDoc)     |
| `npm run format` / `format:check` | Prettier                                                  |
| `npm run typecheck`               | `tsc -b`                                                  |
| `npm test` / `test:watch`         | Vitest + Testing Library                                  |
| `npm run test:coverage`           | Couverture v8, seuils à 100 %                             |
| `npm run mutation`                | Stryker, seuil de score à 100 %                           |

## Notes de version

- **TypeScript 6.0** : `typescript-eslint` ne supporte pas encore TypeScript 7.
- **Vitest 4.1** : le runner Stryker 10 ne fonctionne pas encore avec Vitest 5 (aucun test
  exécuté pendant les runs de mutation).
- `overrides.qs` : force une version corrigée d'une dépendance transitive de Stryker signalée par
  `npm audit`.

Ces épinglages sont à lever dès que les outils concernés sont compatibles (PR Dependabot).

## Docker

```sh
docker build -t edusense/frontend .
docker run --rm -p 8080:8080 edusense/frontend
```

Servi par `nginx-unprivileged` (non root) avec en-têtes de sécurité ([nginx.conf](nginx.conf)).
