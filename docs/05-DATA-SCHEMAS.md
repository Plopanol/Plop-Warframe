# Schémas de données — Plop-Warframe

Autorité pour la base SQLite, ses migrations et le cache des sources. Le DDL fait foi : [`src/db/migrations/`](../src/db/migrations/) ; cette page en donne le sens. Le mapping depuis le wiki est dans [04-REGLES-DROPS](04-REGLES-DROPS.md).

## Emplacement

Dossier de données : `data/` à côté de l'exécutable, ou `--data-dir <dossier>` ([02-ARCHITECTURE](02-ARCHITECTURE.md#données)). En construction de développement (`debug`), le défaut est `data/` à la racine du dépôt, ignoré par Git : il survit à `cargo clean` et garde le cache des modules entre les sessions.

| Chemin | Contenu |
|---|---|
| `plop-warframe.sqlite3` | Base |
| `cache/<module>.lua` | Texte brut du module téléchargé (`?action=raw`) |
| `cache/blueprints_data.lua` (et `.meta`) | Module Lua des recettes, optionnel pour les anciens caches |
| `cache/<module>.lua.meta` | Date de téléchargement (secondes Unix), en-tête `Last-Modified` et, si connue, version du paquet (`version=`) |
| `cache/droptables_de.html` (et `.meta`) | Page des tables officielles de DE, lue pour les reliques ; absente : import sans reliques, avec avertissement |
| `cache/wfcd/<Catégorie>.json`, `cache/wfcd/fr.json` (et `.meta`) | 26 fichiers de catégorie et traduction française de `@wfcd/items` ; le `.meta` ajoute `version=` (en-tête `X-JSD-Version`) ; les fichiers absents donnent un avertissement commun ; `Relics.json` sert aussi au statut vaulted |
| `cache/wfcd_relics.json` (et `.meta`) | Ancien emplacement de `Relics.json`, plus téléchargé ; lu pour le statut vaulted seulement si `cache/wfcd/Relics.json` manque |

## Migrations

`PRAGMA user_version` porte la version du schéma ; la migration `n` (fichier `00n_*.sql`) l'amène de `n-1` à `n`, dans une transaction. Une base plus récente que l'application est refusée. Clés étrangères activées à chaque ouverture.

| Version | Fichier | Contenu |
|---|---|---|
| 1 | `001_initial.sql` | Schéma initial |
| 2 | `002_enemies.sql` | Ennemis, emplacements, ressources de région, tileset et factions des nœuds |
| 3 | `003_relics.sql` | Reliques et leurs récompenses (tables officielles de DE) |
| 4 | `004_relic_vaulted.sql` | Statut vaulted des reliques (WFCD) |
| 5 | `005_wfcd_items.sql` | Items de `@wfcd/items` |
| 6 | `006_recipes.sql` | Recettes du wiki, recettes locales et profil personnage |

## Tables (version 6)

| Table | Clé naturelle | Rôle |
|---|---|---|
| `recipe` | `result` | Recettes du wiki en JSON typé : résultat anglais complet, schéma, quantité produite, crédits et ingrédients |
| `local_recipe` | `result` | Recettes locales prioritaires pour un résultat identique ; conservées au réimport |
| `character_profile` | `id = 1` | Niveau maximal des ennemis positif, ou `NULL` sans plafond ; conservé au réimport |
| `planet` | `name` | Planète, ou `Type` de la table pour une mission spéciale ; `(inconnu)` si vide |
| `mission_type` | `name` | Type de mission (nom anglais du wiki) et `is_endless` |
| `node` | `key` | Nœud ou pseudo-nœud spécial (`is_special`), faction (texte du wiki), niveaux, Secteur sombre, `tileset` |
| `node_faction` | `(node_id, faction)` | Codes de faction normalisés d'un nœud non spécial ([règle 13](04-REGLES-DROPS.md)) |
| `enemy` | `key` | Ennemi de drop (clé de la table, `Heavy Gunner (2)` pour une variante), nom, code de faction ou `NULL` sans emplacement connu |
| `enemy_location` | `(enemy_id, kind, value)` | Restriction du wiki, `kind` ∈ {`planet`, `tileset`} ; aucune ligne : partout où sa faction apparaît |
| `enemy_drop` | — | Taux par kill de 0 à 1 et `rarity` ; plusieurs lignes possibles pour un même objet ; objet `Region Resource` pour une ressource de région |
| `region_resource` | `(planet_id, item_id)` | Ressources d'une planète (`RegionResources`) |
| `drop_table` | `alias` | Table de drop, `table_type` et `tier` du wiki, `rotation_kind` ∈ {`rotation`, `cache`, `stage`, `none`} |
| `node_drop_table` | `(node_id, drop_table_id, role)` | Lien, `role` ∈ {`main`, `cache`, `extra`} |
| `item` | `name` | Objet et catégorie de sa première occurrence (Relic, Mod, Blueprint, Resource…) |
| `drop` | — | Entrée : table, `rotation` ∈ {`A`, `B`, `C`, `NONE`}, objet, `chance` de 0 à 1, `rarity` (code), `quantity` ≥ 1 ; plusieurs lignes possibles pour un même objet (quantités différentes) |
| `relic` | `name` | Relique (« Lith Q3 Relic »), ère (`tier`) et `vaulted` (1 non disponible, 0 disponible, `NULL` inconnu) |
| `relic_reward` | — | Récompense : relique, `refinement` ∈ {`Intact`, `Exceptional`, `Flawless`, `Radiant`}, `item_name` (texte, hors de `item` pour ne pas encombrer la recherche d'objets), `chance` de 0 à 1, `rarity` |
| `wfcd_item` | `unique_name` | Item de `@wfcd/items` ([règle 19](04-REGLES-DROPS.md)) : `category` (fichier source sans `.json`), `name`, `name_fr` (`NULL` sans traduction), `json` (entrée compacte), `json_fr` (traduction compacte ou `NULL`) ; indépendant de `item` |
| `import_run` | — | Historique : début et fin (secondes Unix), source, révision, `status` ∈ {`running`, `success`, `failed`}, message |

Index : `item.name` (unique), `drop(item_id)`, `drop(drop_table_id, rotation)`, `node(mission_type_id)`, `node_drop_table(drop_table_id)`, `node_faction(faction)`, `enemy_drop(item_id)`, `enemy_drop(enemy_id)`, `relic_reward(item_name)`, `relic_reward(relic_id)`, `wfcd_item.unique_name` (unique), `wfcd_item(category)`.

Fonction SQL `fold(texte)`, enregistrée à chaque ouverture : minuscules sans accents latins (« Höllars » → « hollars »), pour la recherche d'objets insensible à la casse et aux accents ; elle n'est pas stockée.

Codes de rareté : `very_common`, `common`, `uncommon`, `rare`, `ultra_rare`, `legendary`, `beyond_legendary`.

## Import

Une seule transaction vide puis recharge les tables importées ; `import_run`, `local_recipe` et `character_profile` sont conservées. Sans `AUTOINCREMENT`, les identifiants repartent de 1, si bien que deux imports des mêmes données donnent des lignes identiques. Une ligne `import_run` est créée avant la transaction (`running`), passée à `success` dans la transaction ou à `failed` avec le message d'erreur après l'annulation : un échec laisse les données précédentes intactes.
