# TASK-001 — Initialiser le projet

- **Statut :** Terminée
- **Phase :** Phase 0 — Cadrage
- **Priorité :** Haute
- **Dépendances :** aucune
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Dépôt organisé selon `_modele-projet` (socle v6, profil complet, langages Rust et Slint, module livrable) : `AGENTS.md`, `CLAUDE.md`, documents numérotés, registres, skills, configuration et outils communs ; cadrage initial archivé.

## Critères d'acceptation

- [x] Déclaration du commun et blocs recopiés par `Sync-Commun.ps1 -Apply`.
- [x] `## Projet`, `## Commandes et contrôles`, documents numérotés et roadmap remplis ; décisions manquantes dans `QUESTIONS.md`.
- [x] Contrôle documentaire exécuté.

## Validation et preuves

- 2026-11-19, Windows : `pwsh -File outils/Sync-Commun.ps1 -Name Plop-Warframe -Apply` depuis `_modele-projet`, code 0.
- 2026-11-19, Windows : `pwsh -NoProfile -File tools/commun/Test-Docs.ps1 -Path .`, résultat consigné dans le commit d'initialisation.

## Obstacles et reprise

Aucun obstacle identifié. L'outil `tools/Test-Projet.ps1` cité par `AGENTS.md` est créé par TASK-003, premier code du dépôt.
