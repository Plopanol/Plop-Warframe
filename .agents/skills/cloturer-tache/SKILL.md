---
name: cloturer-tache
description: Clôture une tâche TASK-XXX selon le socle commun - critères d'acceptation, contrôle complet, preuve datée, statut et index, Prochaine tâche, version et CHANGELOG, commit et fusion dans la branche principale. À utiliser pour clôturer, terminer ou finir une tâche, ou à la fin de son dernier lot.
---

# Clôturer une tâche

Réservé à une tâche en une session ou au dernier lot d'une tâche découpée. Lire la vérification rapide et le contrôle complet dans `## Commandes et contrôles` d'`AGENTS.md`, et les particularités dans `AGENTS.md`, qui prévaut sur ce skill. S'arrêter au premier échec, conserver la branche et rendre compte, parce que la branche principale doit rester saine.

1. Relire la fiche et vérifier chaque critère d'acceptation avec une preuve. Si la tâche touche un écran ou un comportement visible, construire l'application, demander le test à l'utilisateur en listant écrans, actions et résultats attendus, et attendre son retour, qui sert de preuve.
2. Vérifier que les autorités touchées (produit ou GDD, architecture, conventions, ADR, registres) sont à jour dans le changement. Ne modifier `docs/PROJECT-MEMORY.md` que si une route de lecture change, parce que c'est une carte, pas un journal.
3. Si la tâche touche le code, incrémenter le correctif SemVer dans le fichier du champ `Version :` de `## Projet` et ajouter l'entrée de `CHANGELOG.md` ; sinon la version ne change pas.
4. Lancer le contrôle complet une seule fois, sur l'état final du code.
5. Consigner dans `Validation et preuves` de la fiche, datés : contrôles réussis, échoués et non exécutés avec leur raison, validation manuelle éventuelle.
6. Statut : `Terminée` si tous les critères sont satisfaits et prouvés ; `À valider` si une validation attendue manque, et dans ce cas ne pas fusionner. Mettre à jour `Dernière mise à jour` et déplacer la ligne dans la section correspondante de `docs/tasks/INDEX.md`.
7. Mettre à jour la `Prochaine tâche` (section de `docs/03-ROADMAP.md` en profil complet, tête de `docs/tasks/INDEX.md` en profil léger) : un seul lien vers la prochaine tâche exécutable dans l'ordre prévu, ou uniquement « Aucune tâche planifiée. ». Demander à l'utilisateur si le choix est ambigu, parce qu'une priorité inventée égare la session suivante.
8. Lancer la vérification rapide sur l'état final, puis commiter : un commit pour la tâche (celui du dernier lot pour une tâche découpée), Conventional Commit dont le corps référence la TASK, version et `CHANGELOG.md` compris, sans changement étranger.
9. Fusionner localement la branche dans la branche principale ; ne pas pousser.
10. Sur Windows, si la tâche a utilisé Gradle, y compris via Kotlin ou Flutter Android, exécuter `.\gradlew.bat --stop` depuis le dossier du wrapper utilisé, après la dernière commande Gradle. `--stop` agit sur les daemons de la même version, qui peuvent être partagés avec d'autres projets. Si un autre build de cette version est en cours, différer l'arrêt et le signaler dans le rapport. Signaler aussi tout échec de l'arrêt.

Ne déplacer aucune fiche dans `archives/` et ne tenir aucun `JOURNAL.md`, parce que la fiche, l'index et Git portent l'historique.

Rapport : résultat, fichiers, contrôles réussis, échoués et non exécutés, version, commit, fusion, prochaine tâche.

## Points de variation

Le projet les déclare dans `AGENTS.md` (ou, pour les autres commandes, dans `docs/06-CONVENTIONS.md`), sans modifier ce skill :

- la vérification rapide et le contrôle complet (`## Commandes et contrôles` d'`AGENTS.md`) ;
- la commande de construction pour la validation manuelle ;
- un outil de workflow qui exécute ces étapes (par exemple `node tool/agent_workflow.mjs task finish`) ;
- une règle de version plus fine : choix de la composante, document de release, contrôle des notes de version ;
- des revues avant le contrôle complet (par exemple un wiki repéré par son index) ou une purge d'artefacts après la fusion.
