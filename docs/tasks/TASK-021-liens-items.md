# TASK-021 — Liens dans les champs des items

- **Statut :** À valider
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-019
- **Dernière mise à jour :** 2026-11-22
- **Fichiers concernés :** `src/item_view.rs`, `src/ui_bridge/items.rs`, `ui/app-window.slint`, documentation produit.
- **Objectif :** ouvrir dans le navigateur les URL affichées dans les détails des items, notamment `introduced.url`.
- **Travail à réaliser :** détecter les valeurs HTTP/HTTPS dans tous les champs et leur traduction, les afficher comme liens, ouvrir le navigateur par défaut au clic, conserver le tri et signaler les erreurs récupérables.
- **Critères d'acceptation :**
  - [ ] Les valeurs HTTP/HTTPS sont soulignées et cliquables dans les colonnes Valeur et Français, indépendamment du nom du champ.
  - [ ] Le clic ouvre l'adresse dans le navigateur par défaut sans bloquer l'interface ; une erreur de lancement est journalisée et affichée.
  - [ ] Les valeurs ordinaires restent du texte ; le tri reste utilisable.
  - [x] Contrôles rapides, complets et construction debug réussis.
  - [ ] Validation manuelle par l'utilisateur sur Windows.
- **Risques :** navigateur indisponible ; URL invalide dans les données.
- **Tests prévus :** détection des adresses imbriquées et traduites, exclusion des autres protocoles et des adresses invalides ; validation manuelle de l'affichage, du clic et du tri.

## Exécution

Une tranche : détail des items et lancement du navigateur, sans nouvelle dépendance ni téléchargement.

## Validation et preuves

- 2026-11-22, Windows : fmt et Clippy sans avertissement ; 127 tests réussis, dont la détection des URL dans les champs imbriqués et traduits ; 1 test de performance ignoré par défaut (base de développement requise).
- 2026-11-22 : construction release, licences et fraîcheur de NOTICES.md réussies ; contrôles documentaires initialement en échec (entrée de phase manquante, puis tâche à valider non exécutable en « Prochaine tâche »), roadmap corrigée.
- 2026-11-22 : Test-Docs final réussi (4 contrôles, 53 fichiers, 143 liens, aucun lien cassé) ; contrôle complet validé après correction documentaire, sans modification du code après les tests.
- 2026-11-22 : construction debug réussie ; formatage par slint-lsp 1.18.1. Les contrôles suivants utilisent `CARGO_INCREMENTAL=0` pour éviter l'avertissement de liens physiques du cache dans le bac à sable, selon le coffre commun.
- Validation manuelle attendue : lancer `target/debug/plop-warframe.exe`, onglet Items, sélectionner Ash (ou un item comportant `introduced.url` / `wikiUrl`), cliquer le lien souligné de la colonne Valeur ; le navigateur par défaut doit ouvrir l'adresse complète. Trier les trois colonnes puis réessayer ; vérifier les champs ordinaires, le défilement et, si présente, une URL dans la colonne Français.
- Non exécutés : clic réel dans le navigateur (validation utilisateur), essai Linux ; aucun téléchargement effectué.
- Version et changelog à mettre à jour à la clôture après validation manuelle ; branche conservée sans fusion.

## Obstacles et reprise

Aucun obstacle identifié.
