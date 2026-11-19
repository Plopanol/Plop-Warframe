# TASK-013 — Filtre par type d'objet

- **Statut :** Terminée
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-011
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Dans les onglets Recherche Mission et Recherche Spot, une liste « Type » (Mods, Schémas, Ressources, Arcanes, Reliques…) restreint les suggestions d'objets à la catégorie choisie ; champ vide et type choisi : liste des objets de ce type. Le filtre n'agit que sur les suggestions, pas sur les résultats (arbitrage de l'utilisateur, 2026-11-19). Catégorie d'un objet : [05-DATA-SCHEMAS](../05-DATA-SCHEMAS.md) (`item.category`).

## Critères d'acceptation

- [x] Liste « Type » dans les deux onglets, libellés français, premier choix « Tous les types ».
- [x] Suggestions filtrées par type ; champ vide et type choisi : objets du type.
- [x] Tests de requête et de libellés.
- [x] Contrôle complet vert ; validation manuelle par l'utilisateur.

## Réalisation

- `db::search_items` prend une catégorie facultative ; texte vide et catégorie : tous les objets de la catégorie, par nom. `db::item_categories` liste les catégories, les plus fournies d'abord.
- `search_view` : libellés français des catégories (`category_label`), aussi dans la ligne de suggestion ; liste « Tous les types » en tête.
- Les deux onglets : liste « Type » avant le champ Objets, conservée par libellé après un import ; jusqu'à 2 000 suggestions en parcours d'un type, 20 en saisie.

## Validation et preuves

2026-11-19 :

- Contrôle complet (`tools/Test-Projet.ps1 -Complet`) : fmt, Clippy, tests, construction release, licences, `NOTICES.md` réussis ; Test-Docs réussi après indexation de TASK-014 et TASK-015.
- Validation manuelle par l'utilisateur, réussie : liste « Type » dans les deux onglets, parcours d'un type champ vide, saisie filtrée, ajout en puce.
- Contrôle complet de clôture réussi sur l'état final (version 0.1.9).

## Obstacles et reprise

Aucun obstacle identifié.
