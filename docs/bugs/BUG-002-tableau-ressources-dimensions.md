# BUG-002 — Tableau des ressources réduit

- **Statut :** Signalé
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

Dans `ui/recipes.slint`, `FarmTable` est directement enfant de `Tab`, sans conteneur de disposition ; il conserve sa taille préférée. Correction prévue : ajouter un `VerticalBox` et étirer le tableau dans les deux directions, comme les autres vues tabulaires de l'application.

Correction poursuivie sur la branche de TASK-023, dont ces écrans attendent encore la validation utilisateur ; aucune fusion avant cette validation.

## Test de non-régression et contrôles

- Test manuel à effectuer sur l'application reconstruite : afficher les ressources d'une recette, agrandir puis réduire la fenêtre ; le tableau doit remplir la zone disponible, avec défilement des résultats.
- Pas de test d'UI automatisé, conformément au socle d'AGENTS.md.
- Contrôles de construction et de formatage à exécuter ; validation visuelle et contrôle complet restent attendus avant clôture.
