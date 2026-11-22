# BUG-002 — Tableau des ressources réduit

- **Statut :** Corrigé à vérifier
- **Gravité :** Normale
- **Composant :** Interface Recettes
- **Signalé le :** 2026-11-22
- **Version et environnement :** 0.1.19, Windows, branche `task/023-recettes-et-personnage`, commit `eb1992d`
- **Fréquence :** systématique sur la capture fournie
- **Tâche liée :** [TASK-023](../tasks/TASK-023-recettes-et-personnage.md), validation en cours

## Reproduction et preuve

Ouvrir Recettes, sélectionner une recette, calculer les besoins puis afficher « Ressources et pièces ». La capture utilisateur du 2026-11-22 montre un tableau réduit et centré, entouré d'espace inutilisé, avec des colonnes tronquées.

## Résultat attendu

Le tableau remplit en largeur et en hauteur l'espace disponible sous les onglets et suit le redimensionnement de la fenêtre, conformément à la demande utilisateur du 2026-11-22.

## Diagnostic et correction

Dans `ui/recipes.slint`, `FarmTable` était directement enfant de `Tab`, sans conteneur de disposition ; il conservait sa taille préférée. Correction du 2026-11-22 : ajout d'un `VerticalBox` et étirement du tableau dans les deux directions, comme les autres vues tabulaires de l'application.

Correction poursuivie sur la branche de TASK-023, dont ces écrans attendent encore la validation utilisateur ; aucune fusion avant cette validation.

## Test de non-régression et contrôles

- Test manuel à effectuer sur l'application reconstruite : afficher les ressources d'une recette, agrandir puis réduire la fenêtre ; le tableau doit remplir la zone disponible, avec défilement des résultats.
- Pas de test d'UI automatisé, conformément au socle d'AGENTS.md.
- 2026-11-22 : formatage Slint 1.18.1, vérification rapide officielle (fmt, Clippy et tests) et construction debug réussis. Journaux locaux : `target/bug002-rapide.log`, `target/bug002-build.log`.
- Test initial échoué sur la capture utilisateur ; validation du correctif par redimensionnement demandée après reconstruction.
- Validation visuelle, contrôle complet et incrément de version restent attendus avec la clôture de TASK-023 ; pas de fusion à ce stade.
- 2026-11-22 : validation visuelle du tableau reconstruit confirmée par l'utilisateur (« Valider :) »), en réponse à la demande de vérification du remplissage et du redimensionnement. Cette validation ne porte pas sur les autres parcours de TASK-023.
- 2026-11-22 : l'utilisateur valide ensuite explicitement les quatre autres parcours de TASK-023. Contrôle complet de la version 0.1.20 réussi : 136 tests, fmt, Clippy, release, licences, notices et Test-Docs 4/4. Le correctif UI validé est celui du commit `ec4dc09` ; aucun changement de comportement depuis cette validation.
