---
name: signaler-bug
description: Enregistre une anomalie en fiche BUG-XXX avec sa ligne d'index, puis conduit sa confirmation, sa correction et sa fermeture selon docs/bugs/WORKFLOW.md. À utiliser dès qu'un bug, une régression ou un comportement inattendu est signalé, ou pour confirmer, corriger ou fermer un BUG-XXX.
---

# Signaler et corriger un bug

Lire `docs/bugs/WORKFLOW.md`, qui précise le cycle pour le projet, la vérification rapide et le contrôle complet dans `## Commandes et contrôles` d'`AGENTS.md`, et les particularités dans `AGENTS.md`, qui prévaut sur ce skill. Les bugs restent distincts des tâches et ne réordonnent pas la roadmap, sauf dépendance décidée explicitement.

Cycle : `Signalé → Confirmé → En cours → Corrigé à vérifier → Clos`, avec `Bloqué`, `Doublon` ou `Rejeté` si justifié. À chaque changement de statut, modifier la fiche puis régénérer l’index avec la même commande. Vérifier le diff de l’index et inclure les deux fichiers dans le même changement, parce que la fiche fait foi.

## Signaler

1. Prendre le `Prochain identifiant libre` de `docs/bugs/INDEX.md` et l'incrémenter ; ne jamais réutiliser un identifiant.
2. Créer `docs/bugs/BUG-XXX-description.md` depuis le gabarit `docs/bugs/BUG-XXX.md` du projet, statut `Signalé`, même sans cause connue. Aucune donnée sensible, parce que la fiche est versionnée.
3. Depuis la racine du projet, lancer `node tools/commun/check_docs.mjs --fix` en adaptant le chemin au dossier d'outillage déclaré. Vérifier dans le diff que la fiche apparaît sous `Actifs`, puis faire le commit documentaire `docs(bugs): signale BUG-XXX`, dispensé de la vérification rapide du code (le hook pre-commit lance Test-Docs).

## Confirmer et corriger

1. Reproduire sur un état identifié (version, environnement, étapes) avant toute correction ; passer à `Confirmé`.
2. Appliquer les étapes 1 et 2 de `demarrer-tache` (état Git, branches non fusionnées), créer `bug/XXX-description` depuis la branche principale locale et passer à `En cours`.
3. Écrire d'abord le test de non-régression, qui doit échouer, puis corriger, parce qu'un défaut corrigé sans test revient.
4. Si la correction touche le code, incrémenter le correctif SemVer et compléter `CHANGELOG.md` comme à la clôture d'une tâche.
5. Lancer le contrôle complet, consigner dans la fiche correction, test et contrôles datés (réussis, échoués, non exécutés), puis passer à `Corrigé à vérifier`.
6. Commit `fix(portée): …` dont le corps référence le BUG, puis fusion locale dans la branche principale ; ne pas pousser.

## Fermer

1. `Clos` exige une vérification distincte sur l'application reconstruite depuis la branche principale, datée dans la fiche ; un comportement visible se fait vérifier par l'utilisateur. `Corrigé à vérifier` n'est jamais `Clos`.
2. Un défaut conservé volontairement passe à `Clos` avec un lien vers son entrée de `docs/DETTE.md` ; un doublon passe à `Doublon` avec un lien vers la fiche d'origine ; `Rejeté` porte sa justification.
3. Vérifier que la régénération a déplacé la ligne dans la section `Fermés`. Commit `docs(bugs): …`.
4. Un défaut qui persiste rouvre la même fiche, sans nouvel identifiant.

Rapport : identifiant, statut, fichiers, contrôles réussis, échoués et non exécutés, commit.

## Points de variation

Le projet les déclare dans `docs/bugs/WORKFLOW.md` ou dans `AGENTS.md`, sans modifier ce skill :

- le cycle détaillé et les preuves exigées à chaque statut ;
- un outil de workflow (par exemple `node tool/agent_workflow.mjs bug create`, `bug finish`, `bug close`) ;
- la commande de construction utilisée pour la vérification ;
- l'accord de l'utilisateur avant un test qui prend le focus du bureau.
