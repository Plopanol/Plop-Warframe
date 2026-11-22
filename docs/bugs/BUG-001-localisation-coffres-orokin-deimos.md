# BUG-001 — Localisation des coffres Orokin sur Deimos

- **Statut :** Confirmé
- **Gravité :** Normale
- **Composant :** Import wiki et recherche Mission
- **Signalé le :** 2026-11-22
- **Version et environnement :** 0.1.16, Windows ; diagnostic sur le code, les fixtures et le cache local
- **Fréquence :** systématique avec ces données
- **Tâche liée :** aucune

## Reproduction

Dans la recherche Mission, chercher le mod Blind Rage et vérifier sa localisation sur Deimos. Signalement utilisateur ; lecture du chemin d'import et de recherche le 2026-11-22, sans manipulation de l'interface.

## Résultat observé

Deimos manque à la localisation. La table `DerelictVault` est importée sous le pseudo-nœud `special:DerelictVault`, de planète et de type `Orokin Vault`. Elle n'est liée à aucun nœud réel de Deimos ; un filtre qui exclut les missions spéciales l'écarte.

## Résultat attendu

Identifier les coffres Orokin comme source de Blind Rage sur Deimos, avec leurs conditions d'accès et sans présenter leur récompense comme une récompense ordinaire de fin de mission.

Le [wiki, Orokin Vault](https://wiki.warframe.com/w/Orokin_Vault), consulté le 2026-11-22, situe ces coffres sur les missions de Deimos du tileset Orokin Derelict, hors Défense, Assassinat et monde ouvert ; une clé Dragon est requise. La [page Deimos](https://wiki.warframe.com/w/Deimos) donne Blind Rage à 4,17 %.

## Diagnostic

Le Lua contient déjà l'information : `tests/fixtures/droptables_data.lua` et `data/cache/droptables_data.lua` donnent Blind Rage à 4,17 % dans `DerelictVault`, avec `Tier = "Deimos"` et `Type = "Orokin Vault"`. Dans les deux copies de `missions_data.lua`, `MissionTypes["Orokin Vault"].LocationNote` indique tous les nœuds de Deimos sauf Cambion Drift, Magnacidium, Exequias et Hyf.

Dans `src/source/wiki.rs`, `RawMissionType` ne lit pas `LocationNote`. `build_dataset` rattache les tables aux nœuds uniquement par leurs alias explicites ; les coffres n'ont pas de tel lien. La table reste donc une mission spéciale dont la planète reprend le type de table, conformément à la règle actuelle 6 de [04-REGLES-DROPS](../04-REGLES-DROPS.md). `Tier` est conservé sur la table, mais ne sert pas à localiser le pseudo-nœud. `src/db/search.rs` affiche la planète du nœud et peut exclure les missions spéciales.

La divergence établie porte sur l'exploitation de la localisation par l'application, pas sur l'absence du mod dans le Lua. Le téléchargement brut actuel du module n'a pas été vérifié : l'accès web à cette URL n'a pas abouti ; la comparaison Lua repose sur les copies locales.

## Correction et vérification

Aucune correction applicative dans cette demande de vérification. La localisation des récompenses annexes doit conserver la distinction entre récompense de coffre et récompense de mission. Prévoir un test de non-régression sur Blind Rage, les nœuds autorisés et les exclusions avant correction.

Contrôles du 2026-11-22 : comparaison des sources locales et lecture du code réussies ; consultation de la page Orokin Vault réussie. Construction, tests applicatifs et validation manuelle non exécutés (diagnostic documentaire uniquement).
