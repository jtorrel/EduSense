# ADR 0001 — Enregistrer les décisions d'architecture

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

Claude Code ne voit pas les discussions de l'interface web : toute décision doit vivre dans le repo.

## Décision

Chaque décision structurante (choix technique, exclusion de couverture, compromis) fait l'objet d'un ADR `docs/adr/NNNN-<titre>.md` au format Statut / Contexte / Décision / Conséquences. Statuts : Proposé, Accepté, Remplacé par NNNN, Déprécié. Un ADR accepté n'est pas réécrit : on le remplace par un nouveau.

## Conséquences

Historique des décisions traçable ; coût faible de rédaction à chaque décision.
