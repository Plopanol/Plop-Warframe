# Questions ouvertes

Arbitrages en attente. Une question bloquante reste ouverte jusqu'à décision explicite de l'utilisateur ; la décision durable devient une [ADR](adr/INDEX.md) ou une mise à jour de l'autorité concernée. Une question tranchée est retirée de ce fichier : sa décision vit dans l'ADR ou l'autorité concernée, qui cite son identifiant. Prochain identifiant libre : Q-004.

| ID | Sujet | Bloquante | Responsable | État |
|---|---|---|---|---|
| [Q-002](#q-002--version-de-slint-lsp) | Version de `slint-lsp` | Non | Utilisateur | Ouverte |
| [Q-003](#q-003--console-windows-en-release) | Console Windows en release | Non | Utilisateur | Ouverte |

<a id="q-002--version-de-slint-lsp"></a>

## Q-002 — Version de slint-lsp

- Statut : ouverte, non bloquante.
- Constat (TASK-006, 2026-11-19) : `slint-lsp` installé sur le poste est en 1.17.1, alors que le projet épingle Slint 1.18.1 et que le socle demande le formateur de la même version.
- Proposition : `cargo install slint-lsp --version 1.18.1 --locked` ; l'installation modifie le poste et attend l'accord de l'utilisateur.

<a id="q-003--console-windows-en-release"></a>

## Q-003 — Console Windows en release

- Statut : ouverte, non bloquante.
- Constat (TASK-006, 2026-11-19) : l'exécutable est une application console ; lancé par double clic, il ouvre une console à côté de la fenêtre.
- Option : `windows_subsystem = "windows"` en release, au prix de la sortie de `--import`, `--import-cache` et `--stats` dans un terminal. À trancher avant TASK-008.
