# ADR 0011 — Politique d'exclusion de couverture

- **Statut** : Accepté
- **Date** : 2026-10-19

## Contexte

Le seuil de couverture est de 100 %, bloquant. Certains fichiers ne sont pas raisonnablement testables unitairement.

## Décision

Exclusions **autorisées**, et uniquement celles-ci :

1. **Rust** : les fichiers `main.rs` des services, à condition qu'ils se limitent à l'assemblage (lecture de la configuration, initialisation des logs, bind du socket, appel d'une fonction `serve` testée de la bibliothèque, branchement du signal d'arrêt). Mécanisme : alias `cargo cov` (`.cargo/config.toml`), qui passe à `cargo llvm-cov` l'option `--ignore-filename-regex '[\\/]src[\\/]main\.rs$'`. Le séparateur de chemin est explicite : une regex `main\.rs$` exclurait aussi `domain.rs`.
2. **Front** : `src/main.tsx` (montage React dans le DOM), les fichiers de déclaration `*.d.ts`, les fichiers de configuration (`*.config.*`) et, à venir, le code généré (client OpenAPI). Mécanisme : `coverage.exclude` de Vitest.

Toute autre exclusion (attribut `#[coverage(off)]`, commentaire `/* v8 ignore */`, etc.) requiert un nouvel ADR.

## Conséquences

Seuil 100 % tenable sans artifice ; les points d'entrée doivent rester triviaux, ce que la revue vérifie.
