---
description: Phase 4 — traitement de la revue, refactoring et validation sécurité
argument-hint: <XX-nom de l'étape>
---

Phase 4 (refactoring & sécurité) de l'étape `$ARGUMENTS`.

1. Lis `reviews/etape-$ARGUMENTS.md`. Traite chaque constat R-XX, par ordre de sévérité :
   - **traité** : correction appliquée (préciser le commit ou le fichier) ;
   - **rejeté** : justification argumentée.
   Annote le rapport en conséquence (colonne « Statut » + commentaire).
2. Refactoring et optimisation éventuels, sans changer le comportement. Tests toujours verts.
3. Validation sécurité manuelle et consignée en fin de rapport :
   - validation des entrées ;
   - gestion des erreurs sans fuite d'information (RFC 9457) ;
   - authentification / autorisation (si applicable) ;
   - journalisation sans données sensibles ;
   - absence d'`unsafe` (ou ADR).
4. Relance l'ensemble des contrôles de la phase 2 + mutation testing.
5. Commit(s) : `refactor: ...` (et `docs(review): rapport annoté étape $ARGUMENTS`).
6. Pousse la branche et ouvre la PR vers `main` avec : résumé, lien spec, lien revue, checklist des CA. Le merge par l'utilisateur vaut validation.
