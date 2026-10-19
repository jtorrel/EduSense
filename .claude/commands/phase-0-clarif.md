---
description: Phase 0 — clarification de la spec d'une étape
argument-hint: <XX-nom de l'étape, ex. 01-catalogue>
---

Phase 0 (clarification) de l'étape `$ARGUMENTS`.

1. Lis `CLAUDE.md`, `Instructions.md` et `specs/etape-$ARGUMENTS.md`, ainsi que les ADR pertinents de `docs/adr/`.
2. Vérifie que la branche courante est `etape/$ARGUMENTS` ; sinon crée-la depuis `main` à jour.
3. Relis la spec de façon critique et liste :
   - les ambiguïtés et contradictions ;
   - les cas limites non couverts par les critères d'acceptation (entrées invalides, vides, bornes, concurrence, erreurs d'infrastructure) ;
   - les exigences non fonctionnelles manquantes (sécurité, logs, performances) ;
   - les décisions techniques qui mériteraient un ADR.
4. Pose-moi les questions bloquantes **avant** d'écrire quoi que ce soit. Aucune hypothèse silencieuse.
5. Une fois les réponses obtenues, complète la section 8 « Questions / hypothèses » de la spec (et les CA si je l'ai validé), puis crée les ADR nécessaires.
6. Commit : `docs(spec): clarification étape $ARGUMENTS`.
