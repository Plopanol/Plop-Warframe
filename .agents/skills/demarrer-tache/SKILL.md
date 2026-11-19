---
name: demarrer-tache
description: Démarre une tâche TASK-XXX (ou la prochaine tâche planifiée) selon le socle commun - état Git et branches non fusionnées, lecture ciblée depuis PROJECT-MEMORY, dépendances, découpage en lots, branche task/XXX, statut En cours. À utiliser pour lancer, démarrer, attaquer ou reprendre une tâche, ou pour « la prochaine tâche ».
---

# Démarrer une tâche

Lire la vérification rapide et le contrôle complet dans `## Commandes et contrôles` d'`AGENTS.md`, et les particularités dans `AGENTS.md`, qui prévaut sur ce skill.

1. Examiner l'état Git et préserver les changements qui ne viennent pas de l'agent, parce qu'ils appartiennent à l'utilisateur.
2. Lister les branches `task/` et `bug/` non fusionnées dans la branche principale (`git branch --no-merged <principale>`). S'il y en a, demander à l'utilisateur s'il faut la fusionner, la reprendre ou partir de la branche principale, et attendre sa réponse.
3. Identifier la tâche : celle demandée, sinon le lien de la `Prochaine tâche` (`docs/03-ROADMAP.md` en profil complet, tête de `docs/tasks/INDEX.md` en profil léger). Lire sa fiche.
4. Lire `docs/PROJECT-MEMORY.md`, puis seulement la route concernée et les fiches ADR, BUG et questions liées à la tâche, parce que charger toute la documentation coûte sans éclairer.
5. Si la tâche touche la toolchain, le build, le langage, le livrable ou l'outillage agent, lire `../_modele-projet/memory/INDEX.md`, puis seulement les fiches des thèmes concernés, parce que leurs pièges y sont déjà résolus.
6. Vérifier les préconditions : statut `À faire` (`En cours` pour une reprise), dépendances `Terminée`, aucune question bloquante dans `docs/QUESTIONS.md`. Sinon, s'arrêter et rendre compte, parce qu'une décision implicite se découvre trop tard.
7. Découper avant tout code, si nécessaire :
   - profil complet : découper en lots (tableau `Lot | Contenu | Préconditions | Contrôles | État` de la fiche étendue) une tâche qui touche plusieurs couches, de nombreux fichiers, une règle métier complexe, un schéma ou une migration, ou un parcours d'interface ; chaque lot est indépendant, réversible, compilable et vérifiable ; les arbitrages requis précèdent les lots ; pas de lot artificiel pour une petite tâche ;
   - profil léger : une tâche qui exige des lots signale que le projet a grandi ; proposer à l'utilisateur le passage au profil complet.
8. Pour une tâche importante ou une approche non triviale, faire valider le plan par le mode plan de l'outil, parce qu'un plan validé évite de défaire du travail.
9. Créer la branche `task/XXX-description` depuis la branche principale locale ; une correction de bug utilise `bug/XXX-description` (skill `signaler-bug`).
10. Passer la fiche à `En cours`, mettre à jour `Dernière mise à jour` et déplacer sa ligne dans la section `Actives` de `docs/tasks/INDEX.md` ; ces changements entrent dans le commit de la tâche ou du premier lot.

Pendant le travail : vérification rapide avant chaque commit, sauf commit purement documentaire (le hook pre-commit lance alors seulement Test-Docs) ; un commit par lot pour une tâche découpée ; contrôle complet seulement au dernier lot, par `cloturer-tache`, parce qu'il s'exécute une fois par tâche.

Rapport : tâche, branche, lots prévus, contrôles annoncés, questions ouvertes.

## Points de variation

Le projet les déclare dans `## Règles propres au projet` d'`AGENTS.md`, sans modifier ce skill :

- un outil de workflow (par exemple `node tool/agent_workflow.mjs preflight` puis `task start`) : il remplace les étapes 2, 3, 6, 9 et 10, et seuls les fichiers que sa sortie désigne sont lus ;
- le dossier des outils s'il n'est pas `tools/` ;
- des critères de découpage plus précis ;
- une contrainte d'environnement, par exemple travailler dans le checkout principal plutôt que dans un worktree.
