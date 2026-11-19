# TASK-006 — Onglet Données

- **Statut :** À faire
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-004
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Fenêtre Slint à deux onglets ; onglet Données : date, statut et volumes du dernier import, chemin de la base, boutons « Mettre à jour depuis le wiki » et « Réimporter depuis le cache », barre de progression et journal ; import dans un thread de fond, retour par `slint::invoke_from_event_loop`.

## Critères d'acceptation

- [ ] L'interface reste réactive pendant un import complet.
- [ ] Validation manuelle demandée à l'utilisateur (écrans, actions, résultats attendus).

## Validation et preuves

Non exécutée.

## Obstacles et reprise

Aucun obstacle identifié.
