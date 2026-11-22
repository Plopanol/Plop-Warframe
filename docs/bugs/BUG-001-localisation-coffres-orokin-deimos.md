# BUG-001 — Localisation des coffres Orokin sur Deimos

- **Statut :** Corrigé à vérifier
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

Correction du 2026-11-22, version 0.1.17 : `DerelictVault` est liée en rôle `extra` aux cinq nœuds compatibles de Deimos du tileset Orokin Derelict (Capture, Exterminate, Sabotage, Mobile Defense, Survival). La table doit exister et ne pas être ignorée ; aucun lien en double n'est ajouté. Les autres nœuds et tilesets sont exclus. Le pseudo-nœud spécial disparaît quand les nœuds réels utilisent la table. Le pool s'affiche « Coffre Orokin (clé Dragon) » ; le taux reste propre au coffre, sans rotation ni chance AABC. Cette correction s'applique à tous les objets de la table.

Tests de non-régression : `blind_rage_is_found_in_deimos_vaults_without_special_missions` dans `tests/import_tests.rs` vérifie le parcours import → SQLite → recherche par objet et combinée, les cinq nœuds exacts, le taux, le rôle annexe, l'absence de rotation/AABC et le filtre Capture (Horend seul). `orokin_vault_results_explain_the_dragon_key_requirement` dans `src/search_view.rs` vérifie le texte du pool. Les deux tests ont échoué avant correction le 2026-11-22 (aucun résultat et ancien libellé) ; le test d'import ciblé passe après correction.

Contrôles du 2026-11-22 : `pwsh -NoProfile -File tools/Test-Projet.ps1 -Complet` réussi avec `CARGO_INCREMENTAL=0` (contournement documenté du refus de liens physiques du cache incrémental dans le bac à sable) : formatage, Clippy sans avertissement, 122 tests réussis, construction release, licences, notices à jour et quatre contrôles documentaires réussis. Le test de performance sur base de développement reste ignoré par défaut. Les premières passes ont détecté les attentes de comptage de l'ancien pseudo-nœud ; elles ont été actualisées (523 nœuds au lieu de 524), puis les contrôles ont réussi. Aucun échec restant.

Validation manuelle par l'utilisateur non exécutée ; le bug reste `Corrigé à vérifier` jusqu'à ce retour. La base existante doit être réimportée depuis le cache avec l'exécutable reconstruit pour recevoir les nouveaux liens ; aucun téléchargement n'est nécessaire.

Parcours manuel attendu sur l'exécutable reconstruit et la base réimportée depuis le cache : onglet Mission, sélectionner Blind Rage, tous types de mission, missions spéciales exclues → Dirus, Formido, Horend, Phlegyas, Terrorem sur Deimos à 4,17 %, pool « Coffre Orokin (clé Dragon) », aucune chance AABC ; filtre Capture → Horend seul ; ni Hyf, ni Magnacidium, ni Exequias, ni Cambion Drift dans les résultats. Vérifier les modes par objet et combiné.
