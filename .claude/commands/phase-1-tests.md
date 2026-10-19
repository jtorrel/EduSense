---
description: Phase 1 — écriture des tests (rouges) depuis les critères d'acceptation
argument-hint: <XX-nom de l'étape>
---

Phase 1 (tests) de l'étape `$ARGUMENTS`.

1. Relis `specs/etape-$ARGUMENTS.md` (section 3 et section 8 incluses) et `CLAUDE.md`.
2. Pour **chaque** critère d'acceptation CA-XX, écris les tests correspondants :
   - nom ou doc du test référençant le CA (`ca01_...`, `/// CA-01 ...`, `it('CA-01 ...')`) ;
   - niveau adapté : unitaire (domaine, handlers avec mocks `mockall`), intégration (`testcontainers`), composant (Testing Library) ;
   - cas nominaux **et** cas limites identifiés en phase 0.
3. Ajoute uniquement les signatures / stubs nécessaires pour que les tests **compilent** (`todo!()` / `unimplemented!()` côté Rust, fonctions vides côté TS). Aucune logique.
4. Exécute les tests et vérifie qu'ils **échouent** pour la bonne raison. Montre-moi la sortie.
5. Commit : `test: tests de l'étape $ARGUMENTS (rouges)`. Ce commit est la preuve du TDD.
