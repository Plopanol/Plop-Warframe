# TASK-021 — Liens dans les champs des items

- **Statut :** Terminée
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-019
- **Dernière mise à jour :** 2026-11-22
- **Fichiers concernés :** `src/item_view.rs`, `src/ui_bridge/items.rs`, `ui/app-window.slint`, documentation produit.
- **Objectif :** ouvrir dans le navigateur les URL affichées dans les détails des items, notamment `introduced.url`.
- **Travail à réaliser :** détecter les valeurs HTTP/HTTPS dans tous les champs et leur traduction, les afficher comme liens, ouvrir le navigateur par défaut au clic, conserver le tri et signaler les erreurs récupérables.
- **Critères d'acceptation :**
  - [x] Le titre affiche « Détail : NOM – URL », avec `wikiaUrl` cliquable à côté du nom ; sans adresse valide, seul le nom apparaît.
  - [x] Les valeurs HTTP/HTTPS sont soulignées et cliquables dans les colonnes Valeur et Français, indépendamment du nom du champ.
  - [x] Le clic ouvre l'adresse dans le navigateur par défaut sans bloquer l'interface ; une erreur de lancement est journalisée et affichée.
  - [x] Les valeurs ordinaires restent du texte ; le tri reste utilisable.
  - [x] Contrôles rapides, complets et construction debug réussis.
  - [x] Validation manuelle par l'utilisateur sur Windows.
- **Risques :** navigateur indisponible ; URL invalide dans les données.
- **Tests prévus :** détection des adresses imbriquées et traduites, exclusion des autres protocoles et des adresses invalides ; validation manuelle de l'affichage, du clic et du tri.

## Exécution

Une tranche : détail des items et lancement du navigateur, sans nouvelle dépendance ni téléchargement.

Complément demandé le 2026-11-22 : afficher aussi `wikiaUrl` dans l'en-tête, à côté du nom de l'item ; réutiliser l'ouverture du navigateur des liens du tableau. Le lien est effacé au changement de recherche ou pour un item sans URL valide.

## Validation et preuves

- 2026-11-22, Windows : fmt et Clippy sans avertissement ; 127 tests réussis, dont la détection des URL dans les champs imbriqués et traduits ; 1 test de performance ignoré par défaut (base de développement requise).
- 2026-11-22 : construction release, licences et fraîcheur de NOTICES.md réussies ; contrôles documentaires initialement en échec (entrée de phase manquante, puis tâche à valider non exécutable en « Prochaine tâche »), roadmap corrigée.
- 2026-11-22 : Test-Docs final réussi (4 contrôles, 53 fichiers, 143 liens, aucun lien cassé) ; contrôle complet validé après correction documentaire, sans modification du code après les tests.
- 2026-11-22 : construction debug réussie ; formatage par slint-lsp 1.18.1. Les contrôles suivants utilisent `CARGO_INCREMENTAL=0` pour éviter l'avertissement de liens physiques du cache dans le bac à sable, selon le coffre commun.
- Validation manuelle attendue : lancer `target/debug/plop-warframe.exe`, onglet Items, sélectionner Ash (ou un item comportant `introduced.url` / `wikiUrl`), cliquer le lien souligné de la colonne Valeur ; le navigateur par défaut doit ouvrir l'adresse complète. Trier les trois colonnes puis réessayer ; vérifier les champs ordinaires, le défilement et, si présente, une URL dans la colonne Français.
- Non exécutés : clic réel dans le navigateur (validation utilisateur), essai Linux ; aucun téléchargement effectué.
- Complément d'en-tête : sélectionner un item avec `wikiaUrl`, vérifier « Détail : NOM – URL » et cliquer l'adresse ; changer d'item et de recherche pour vérifier que le lien suit la sélection et disparaît sans adresse valide. Les liens du tableau restent disponibles.
- 2026-11-22, complément d'en-tête : contrôle complet et construction debug réussis sans avertissement, avec `CARGO_INCREMENTAL=0` (fmt, Clippy, 128 tests réussis et 1 test de performance ignoré, release, licences, notices et 4 contrôles documentaires). `detail_header_uses_only_a_valid_wikia_url` couvre l'adresse de l'item, son absence, une valeur nulle et un protocole non web. Validation manuelle toujours attendue.
- 2026-11-22 : validation manuelle de l'utilisateur (« Je valide :) ») ; critères d'affichage et d'ouverture acceptés. Clôture en 0.1.18 ; version et changelog mis à jour.
- 2026-11-22 : contrôle complet de clôture en 0.1.18 réussi (fmt, Clippy, 128 tests, release, licences, notices, documentation). Le premier essai avait rencontré l'exécutable debug ouvert (`os error 5`) ; instance du dépôt fermée normalement après vérification du chemin, selon la leçon du coffre `environnement/exe-verrouille-windows.md`. L'application installée est préservée.

## Obstacles et reprise

Aucun obstacle identifié.
