# Schémas de données — Plop-Warframe

Autorité pour la base SQLite, ses migrations et le cache des sources. Le DDL fait foi : [`src/db/migrations/`](../src/db/migrations/) ; cette page en donne le sens. Le mapping depuis le wiki est dans [04-REGLES-DROPS](04-REGLES-DROPS.md).

## Emplacement

Dossier de données : `data/` à côté de l'exécutable, ou `--data-dir <dossier>` ([02-ARCHITECTURE](02-ARCHITECTURE.md#données)).

| Chemin | Contenu |
|---|---|
| `plop-warframe.sqlite3` | Base |
| `cache/<module>.lua` | Texte brut du module téléchargé (`?action=raw`) |
| `cache/<module>.lua.meta` | Date de téléchargement (secondes Unix) et en-tête `Last-Modified` |

## Migrations

`PRAGMA user_version` porte la version du schéma ; la migration `n` (fichier `00n_*.sql`) l'amène de `n-1` à `n`, dans une transaction. Une base plus récente que l'application est refusée. Clés étrangères activées à chaque ouverture.

| Version | Fichier | Contenu |
|---|---|---|
| 1 | `001_initial.sql` | Schéma initial |

## Tables (version 1)

| Table | Clé naturelle | Rôle |
|---|---|---|
| `planet` | `name` | Planète, ou `Type` de la table pour une mission spéciale ; `(inconnu)` si vide |
| `mission_type` | `name` | Type de mission (nom anglais du wiki) et `is_endless` |
| `node` | `key` | Nœud ou pseudo-nœud spécial (`is_special`), faction, niveaux, Secteur sombre |
| `drop_table` | `alias` | Table de drop, `table_type` et `tier` du wiki, `rotation_kind` ∈ {`rotation`, `cache`, `stage`, `none`} |
| `node_drop_table` | `(node_id, drop_table_id, role)` | Lien, `role` ∈ {`main`, `cache`, `extra`} |
| `item` | `name` | Objet et catégorie de sa première occurrence (Relic, Mod, Blueprint, Resource…) |
| `drop` | — | Entrée : table, `rotation` ∈ {`A`, `B`, `C`, `NONE`}, objet, `chance` de 0 à 1, `rarity` (code), `quantity` ≥ 1 ; plusieurs lignes possibles pour un même objet (quantités différentes) |
| `import_run` | — | Historique : début et fin (secondes Unix), source, révision, `status` ∈ {`running`, `success`, `failed`}, message |

Index : `item.name` (unique), `drop(item_id)`, `drop(drop_table_id, rotation)`, `node(mission_type_id)`, `node_drop_table(drop_table_id)`.

Codes de rareté : `very_common`, `common`, `uncommon`, `rare`, `ultra_rare`, `legendary`, `beyond_legendary`.

## Import

Une seule transaction vide puis recharge toutes les tables sauf `import_run` ; sans `AUTOINCREMENT`, les identifiants repartent de 1, si bien que deux imports des mêmes données donnent des lignes identiques. Une ligne `import_run` est créée avant la transaction (`running`), passée à `success` dans la transaction ou à `failed` avec le message d'erreur après l'annulation : un échec laisse les données précédentes intactes.
