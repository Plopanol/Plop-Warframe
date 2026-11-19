# TASK-004 — Base SQLite et import transactionnel

- **Statut :** À faire
- **Phase :** Phase 1 — Données
- **Priorité :** Haute
- **Dépendances :** TASK-003, [Q-001](../QUESTIONS.md#q-001)
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Schéma normalisé (`planet`, `mission_type`, `node`, `drop_table`, `node_drop_table` avec `role`, `item`, `drop`, `import_run`), clés étrangères, index, migrations par `PRAGMA user_version` ; import idempotent en une transaction ; téléchargement des modules avec User-Agent, délai et cache disque ; CLI `--import` et `--import-cache`. Schéma documenté dans `docs/05-DATA-SCHEMAS.md`.

## Critères d'acceptation

- [ ] Deux imports successifs donnent le même contenu ; `import_run` conservé.
- [ ] Import depuis le cache sans réseau.
- [ ] Emplacement de la base et du cache conforme à l'arbitrage de Q-001.

## Validation et preuves

Non exécutée.

## Obstacles et reprise

Q-001 (emplacement de la base) doit être tranchée avant le code d'ouverture de la base et du cache ; le schéma et l'import peuvent avancer sur une base en mémoire.
