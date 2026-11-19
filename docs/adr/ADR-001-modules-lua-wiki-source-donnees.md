# ADR-001 — Modules Lua du wiki comme source

- **Statut :** Acceptée
- **Date :** 2026-11-19
- **Source de la décision :** utilisateur, validation du compte rendu de l'étape 0 (TASK-002)
- **Remplace :** aucune

## Contexte

Les tables de drop du wiki Warframe sont générées par des modules Lua MediaWiki. L'étape 0 a téléchargé `Module:DropTables/data`, `Module:Missions/data` et la page `Drop_Tables` : les deux modules sont du Lua pur, sans `require` ni appel `mw.*` actif, avec un post-traitement Lua (`table.sort`, index inversé `Rewards`) avant le `return` ; la page `Drop_Tables` ne contient aucune table de drop, seulement des explications.

## Décision

Exécuter les deux modules dans un interpréteur Lua 5.4 embarqué (`mlua`, `vendored`) et désérialiser la table retournée par serde. L'interpréteur n'ouvre que les bibliothèques sûres (`table`, `string`, `math`, `utf8`), sans `io`, `os`, `package` ni `debug`, parce que le code exécuté vient d'un wiki modifiable. Le parseur reste derrière le trait `DropSource`.

## Alternatives et conséquences

- Analyse du HTML rendu (`scraper`) : écartée, la page ne contient pas les données.
- Analyse syntaxique maison du Lua : écartée, le post-traitement du module serait à réimplémenter et casserait à chaque évolution.
- Tables officielles de DE (`warframe.com/droptables`) : source future possible derrière `DropSource`.
- Conséquence : dépendance à un compilateur C pour la compilation de Lua (`vendored`) ; MSVC sous Windows, `cc` sous Linux.

## Autorités concernées

[02-ARCHITECTURE](../02-ARCHITECTURE.md), [04-REGLES-DROPS](../04-REGLES-DROPS.md).
