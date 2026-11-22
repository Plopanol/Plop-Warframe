# Roadmap — Plop-Warframe

Autorité pour l'ordre des phases et des tâches ; elle ne porte jamais de statut. Les statuts sont dans [l'index des tâches](tasks/INDEX.md).

## Phase 0 — Cadrage

| Ordre | Livrable | Dépendances |
|---|---|---|
| [TASK-001](tasks/TASK-001-initialiser-le-projet.md) | Dépôt organisé selon le socle commun v6 | Aucune |
| [TASK-002](tasks/TASK-002-analyser-sources-wiki.md) | Fixtures du wiki, structure et mapping validés | Aucune |

Jalon : mapping des sources validé par l'utilisateur.

## Phase 1 — Données

| Ordre | Livrable | Dépendances |
|---|---|---|
| [TASK-003](tasks/TASK-003-parseur-modules-lua.md) | Parseur Lua des modules et tests sur fixtures | TASK-002 |
| [TASK-004](tasks/TASK-004-base-sqlite-import.md) | Schéma SQLite, migrations, import transactionnel, CLI `--import` | TASK-003 |
| [TASK-009](tasks/TASK-009-donnees-de-developpement.md) | Données de développement conservées dans `data/` du dépôt | TASK-004 |
| [TASK-005](tasks/TASK-005-requetes-recherche.md) | Requêtes de recherche par objet et combinée, tests | TASK-004 |

Jalon : `--import` remplit la base depuis le wiki et une requête de test retrouve les valeurs du wiki.

## Phase 2 — Interface

| Ordre | Livrable | Dépendances |
|---|---|---|
| [TASK-006](tasks/TASK-006-onglet-donnees.md) | Onglet Données, import en arrière-plan | TASK-004 |
| [TASK-010](tasks/TASK-010-console-release.md) | Pas de console Windows en release (Q-003) | TASK-006 |
| [TASK-007](tasks/TASK-007-onglet-recherche.md) | Onglet Recherche | TASK-005, TASK-006 |
| [TASK-011](tasks/TASK-011-onglet-recherche-spot.md) | Onglet Recherche Spot (drops des ennemis) | TASK-007 |
| [TASK-008](tasks/TASK-008-finitions.md) | Gestion d'erreurs, README, notices, livrable | TASK-011 |
| [TASK-013](tasks/TASK-013-filtre-type-objet.md) | Filtre des suggestions par type d'objet | TASK-011 |
| [TASK-014](tasks/TASK-014-theme-wiki.md) | Couleurs et polices du wiki | TASK-008 |
| [TASK-015](tasks/TASK-015-icone-coffre.md) | Icône coffre de loot (fenêtre et exe) | TASK-008 |
| [TASK-016](tasks/TASK-016-secteurs-sombres.md) | Badge et filtre Secteurs sombres | TASK-007 |
| [TASK-017](tasks/TASK-017-reliques-warframe.md) | Onglet Reliques d'une Warframe (tables DE) | TASK-011 |
| [TASK-018](tasks/TASK-018-reliques-vaulted.md) | Colonne Vaulted des reliques (WFCD) | TASK-017 |
| [TASK-019](tasks/TASK-019-items-wfcd.md) | Items du jeu WFCD en cache et en base | TASK-018 |
| [TASK-020](tasks/TASK-020-onglet-carte-stellaire.md) | Onglet Carte stellaire (nœuds par type et niveau) | TASK-016 |
| [TASK-021](tasks/TASK-021-liens-items.md) | Liens dans les champs des items | TASK-019 |
| [TASK-022](tasks/TASK-022-nettoyage-repo.md) | Nettoyage local sécurisé des sorties Rust | TASK-021 |
| [TASK-023](tasks/TASK-023-recettes-et-personnage.md) | Profil personnage, recettes imbriquées et spots par ingrédient | TASK-019, TASK-020 |
| [TASK-012](tasks/TASK-012-controle-linux.md) | Contrôle Linux (reporté) | TASK-008 |

Jalon : critères de succès de [01-PRODUIT](01-PRODUIT.md) vérifiés sous Windows et Linux.

## Prochaine tâche

[TASK-023 — Recettes et profil personnage](tasks/TASK-023-recettes-et-personnage.md)
