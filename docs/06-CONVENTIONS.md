# Conventions — Plop-Warframe

Autorité pour le style, les tests, les autres commandes et la définition de « terminé ». Les règles communes (statuts, priorités, nommage, index, Git, langue) sont dans le socle d'[AGENTS.md](../AGENTS.md) et ne sont pas recopiées ici.

## Documents

Numérotation du modèle Plop : `01-PRODUIT`, `02-ARCHITECTURE`, `03-ROADMAP`, `04-REGLES-DROPS` (lecture des sources du wiki), `05-DATA-SCHEMAS` (schéma SQLite, créé par TASK-004), `06-CONVENTIONS`.

## Fiche de tâche

- `TASK-XXX-simple.md` du modèle : tâche courte en une session.
- `TASK-XXX-etendue.md` du modèle : champs en tête (Statut, Phase, Priorité, Dépendances, Fichiers concernés, Objectif, Travail à réaliser, Critères d'acceptation, Risques, Tests prévus), puis `Exécution`, `Journal des lots`, `Validation et preuves`, `Obstacles et reprise`.

Le découpage en lots est défini par le profil complet d'[AGENTS.md](../AGENTS.md) ; tableau `Lot | Contenu | Préconditions | Contrôles | État`.

## Style de code

`rustfmt.toml`, `clippy.toml` et `[workspace.lints]` de `Cargo.toml` font foi. Erreurs du cœur par `thiserror` ; `anyhow` réservé au binaire.

## Stratégie de tests

- Tests d'intégration dans `tests/` : `parse_tests.rs` sur les fixtures du wiki, `query_tests.rs` sur une base en mémoire.
- Aucun test n'accède au réseau ; les fixtures de `tests/fixtures/` se rafraîchissent à la main (voir son `README.md`) et les valeurs attendues des tests se revérifient sur le wiki à cette occasion.
- Un correctif de bug reçoit un test de non-régression lorsque c'est techniquement possible.

## Commandes et contrôles

### Autres commandes

Lancement, construction, outils. La vérification rapide et le contrôle complet sont dans [AGENTS.md](../AGENTS.md).

| Usage | Commande |
|---|---|
| Lancer l'application | `cargo run` |
| Importer depuis le wiki, depuis le cache, état de la base | `cargo run -- [--data-dir <dossier>] --import`, `--import-cache`, `--stats` |
| Construire | `cargo build --release --locked` |
| Licences des dépendances | `cargo deny check licenses` |

## Commits

Conventional Commits du socle. Portées en usage : `source`, `db`, `ui`, `socle`, `outils`.

## Définition de « terminé »

Tous les critères d'acceptation cochés, contrôles prévus exécutés avec résultat consigné et daté dans la fiche, index, roadmap (`Prochaine tâche`) et mémoire synchronisés, commit effectué.
