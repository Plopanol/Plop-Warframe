---
name: nouvelle-adr
description: Acte une décision durable et structurante en fiche ADR-XXX avec sa ligne d'index, met à jour les ADR remplacées et retire la question tranchée de docs/QUESTIONS.md. À utiliser pour acter ou documenter une décision, créer une ADR, ou quand l'utilisateur tranche une question Q-XXX.
---

# Nouvelle ADR

Réserver une ADR aux choix durables et structurants, parce qu'un registre de micro-décisions ne se lit plus. Un arbitrage ponctuel va dans la fiche TASK ou BUG, une règle de produit ou de jeu dans `docs/01-PRODUIT.md` ou `docs/01-GDD.md`, un ordre de travail dans la roadmap. Lire les commandes dans `## Commandes et contrôles` d'`AGENTS.md`, et les particularités dans `AGENTS.md`, qui prévaut sur ce skill.

Avant d'écrire :

- Sans arbitrage de l'utilisateur, ouvrir une question dans `docs/QUESTIONS.md` et s'arrêter, parce qu'une hypothèse n'est pas une décision.
- Ne pas rediscuter une ADR acceptée sans information nouvelle ou blocage démontré.

1. Prendre le `Prochain identifiant libre` de `docs/adr/INDEX.md` et l'incrémenter ; ne jamais réutiliser un numéro, même supprimé.
2. Créer `docs/adr/ADR-XXX-description.md` depuis le gabarit `docs/adr/ADR-XXX.md` du projet : titre de 8 mots au plus, statut `Acceptée` seulement si l'utilisateur a tranché (sinon `Proposée`), date, source de la décision, ADR remplacée ; contexte, décision, alternatives et conséquences en quelques lignes.
3. Ajouter la ligne d'index : identifiant lié, titre, statut, décision courte.
4. Passer une ADR remplacée à `Remplacée par ADR-XXX`, dans sa fiche et dans l'index.
5. Mettre à jour les autorités concernées par un renvoi à l'ADR, sans recopier son contenu, parce qu'une règle recopiée diverge de sa source.
6. Si l'ADR tranche une question : citer `Q-XXX` dans la source de la décision, retirer la ligne et la section de la question de `docs/QUESTIONS.md`, ne jamais réutiliser son identifiant, et ajouter l'entrée à `docs/archive/QUESTIONS-RESOLUES.md` (historique non normatif, créé au besoin et référencé dans `docs/archive/INDEX.md`).
7. Dans une tâche en cours, l'ADR entre dans le commit de la tâche, avec ses contrôles ; sinon, commit documentaire distinct `docs(adr): ajoute ADR-XXX` sur la branche principale, dispensé de la vérification rapide du code (le hook pre-commit lance Test-Docs).

Rapport : identifiant, titre, statut, autorités modifiées, question retirée, contrôles, commit.

## Points de variation

Le projet les déclare dans `AGENTS.md`, sans modifier ce skill :

- un outil de workflow (par exemple `node tool/agent_workflow.mjs adr create`) ;
- un contrôle allégé pour un changement documentaire seul ;
- des champs d'ADR propres au projet.
