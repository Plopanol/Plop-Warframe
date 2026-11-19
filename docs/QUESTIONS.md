# Questions ouvertes

Arbitrages en attente. Une question bloquante reste ouverte jusqu'à décision explicite de l'utilisateur ; la décision durable devient une [ADR](adr/INDEX.md) ou une mise à jour de l'autorité concernée. Une question tranchée est retirée de ce fichier : sa décision vit dans l'ADR ou l'autorité concernée, qui cite son identifiant. Prochain identifiant libre : Q-004.

| ID | Sujet | Bloquante | Responsable | État |
|---|---|---|---|---|
| [Q-002](#q-002--version-de-slint-lsp) | Version de `slint-lsp` | Non | Utilisateur | Ouverte |

<a id="q-002--version-de-slint-lsp"></a>

## Q-002 — Version de slint-lsp

- Statut : ouverte, non bloquante.
- Constat (TASK-006, 2026-11-19) : `slint-lsp` installé sur le poste est en 1.17.1, alors que le projet épingle Slint 1.18.1 et que le socle demande le formateur de la même version.
- Accord de principe de l'utilisateur le 2026-11-19 pour se mettre à jour.
- Impact : `slint-lsp` est un binaire unique pour tout le poste, et Plop-Ogame et Plop-Manga épinglent encore Slint 1.17.1 ; le remplacer décalerait leur formatage. Reste à choisir entre remplacer le binaire global et installer la 1.18.1 à côté (`--root`).
