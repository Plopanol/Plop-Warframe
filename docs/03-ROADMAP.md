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
| [TASK-008](tasks/TASK-008-finitions.md) | Gestion d'erreurs, README, contrôle Linux | TASK-011 |

Jalon : critères de succès de [01-PRODUIT](01-PRODUIT.md) vérifiés sous Windows et Linux.

## Prochaine tâche

[TASK-011](tasks/TASK-011-onglet-recherche-spot.md)
