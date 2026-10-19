---
description: Phase 3 — revue critique indépendante par un sous-agent neuf
argument-hint: <XX-nom de l'étape>
---

Phase 3 (revue critique) de l'étape `$ARGUMENTS`.

1. Produis le diff de l'étape : `git diff main...HEAD > /tmp/etape-$ARGUMENTS.diff` (adapter le chemin temporaire).
2. Lance un **sous-agent neuf** (outil Agent, type `general-purpose`) qui ne reçoit **que** :
   - le chemin de la spec `specs/etape-$ARGUMENTS.md` ;
   - le chemin du diff ;
   - la consigne ci-dessous. Ne lui transmets aucun élément de contexte de cette session.

   > Tu es relecteur indépendant. Lis la spec et le diff (et les fichiers du repo si nécessaire pour comprendre le diff). Évalue : conformité aux critères d'acceptation, qualité et pertinence des tests (assertions réelles, cas limites, traçabilité CA), correction du code, sécurité (validation des entrées, fuites d'information, logs, `unsafe`), documentation, respect de `CLAUDE.md`. Exécute `cargo mutants --in-diff` sur le diff et `npm run mutation` dans `frontend/` si le front est modifié, et rapporte les mutants survivants. Classe chaque constat par sévérité : **bloquant**, **majeur**, **mineur**, **suggestion**, avec fichier:ligne et proposition de correction. N'invente pas de problème : si un point est conforme, ne le signale pas.

3. Écris le rapport du sous-agent dans `reviews/etape-$ARGUMENTS.md` (un identifiant R-XX par constat, colonne « Statut » vide).
4. Ne corrige rien à cette phase.
5. Commit : `review: rapport de revue étape $ARGUMENTS`.
