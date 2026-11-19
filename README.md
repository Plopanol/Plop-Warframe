# Plop-Warframe

Application de bureau qui répond à la question « où farmer cet objet ? » dans Warframe. Elle importe les tables de drop du [wiki officiel](https://wiki.warframe.com/) dans une base SQLite locale, puis indique, pour un type de mission et des objets choisis, les nœuds et les ennemis au meilleur taux de drop, consultables hors ligne.

Interface en français ; Windows (Linux prévu). Détail du produit : [docs/01-PRODUIT.md](docs/01-PRODUIT.md).

## Fonctionnalités

- **Données** : import des modules Lua du wiki (`Module:DropTables/data`, `Module:Missions/data`, `Module:Enemies/data/<faction>`), réimport depuis le cache sans réseau, historique des imports.
- **Recherche Mission** : pour un type de mission (Survie, Défense, Excavation…) et des objets, nœuds triés par chance, avec filtres de rotation et de missions spéciales (primes, Sortie, Arbitrages, Cauchemar…) ; mode par objet ou combiné, chance par cycle AABC des missions sans fin.
- **Recherche Spot** : ennemis au meilleur taux par kill pour les objets choisis, puis nœuds où ils apparaissent, filtrés par type de mission et planète.

Les noms d'objets et de nœuds restent en anglais, comme dans le jeu.

## Utiliser la version Windows

1. Décompresser `plop-warframe-<version>-x86_64-pc-windows-msvc.zip` dans un dossier où l'on peut écrire.
2. Lancer `plop-warframe.exe`.
3. Au premier lancement, onglet **Données** : **Mettre à jour depuis le wiki** télécharge les modules du wiki et remplit la base (une requête par seconde, pour ménager le wiki).
4. Onglets **Recherche Mission** et **Recherche Spot** : choisir les objets et les filtres.

L'application n'accède au réseau que sur ce bouton. Les erreurs (réseau, base occupée ou endommagée, cache absent) s'affichent dans le journal de l'onglet Données ou dans le résumé de l'onglet de recherche.

## Emplacement de la base

Tout l'état de l'application est dans le dossier `data/` à côté de l'exécutable :

| Fichier | Contenu |
|---|---|
| `data/plop-warframe.sqlite3` | Base SQLite des drops |
| `data/cache/` | Modules Lua du wiki téléchargés, pour réimporter sans réseau |

Déplacer ou sauvegarder le dossier de l'application suffit ; le supprimer efface tout. L'option `--data-dir <dossier>` choisit un autre dossier. Schéma de la base : [docs/05-DATA-SCHEMAS.md](docs/05-DATA-SCHEMAS.md).

## Mettre à jour

- **Les données** : bouton **Mettre à jour depuis le wiki** (nouveau téléchargement) ou **Réimporter depuis le cache** (sans réseau). L'import est transactionnel : un échec laisse la base précédente intacte.
- **L'application** : remplacer `plop-warframe.exe` par celui de la nouvelle archive en conservant `data/`. Une base plus ancienne est migrée automatiquement à l'ouverture ; une base créée par une version plus récente est refusée avec un message.

## Compiler depuis les sources

Prérequis : la toolchain Rust épinglée par `rust-toolchain.toml` (installée par `rustup`), un compilateur C (MSVC sous Windows) pour SQLite et Lua embarqués, PowerShell 7 pour les scripts de `tools/`.

```bash
cargo build --release --locked
```

L'exécutable est `target/release/plop-warframe.exe`. En construction de développement (`cargo run`), le dossier de données par défaut est `data/` à la racine du dépôt. Les tests (`cargo test`) tournent hors ligne sur les copies figées de `tests/fixtures/`. Autres commandes (import en ligne de commande, licences) : [docs/06-CONVENTIONS.md](docs/06-CONVENTIONS.md) ; contrôles : `## Commandes et contrôles` d'[AGENTS.md](AGENTS.md) ; production de l'archive : [docs/RELEASE.md](docs/RELEASE.md).

## Contribuer

Le dépôt suit les règles d'[AGENTS.md](AGENTS.md) : une fiche par tâche dans [docs/tasks/](docs/tasks/INDEX.md), une par anomalie dans [docs/bugs/](docs/bugs/INDEX.md), décisions dans [docs/adr/](docs/adr/INDEX.md). Point d'entrée de la documentation : [docs/PROJECT-MEMORY.md](docs/PROJECT-MEMORY.md).

## Licences et mentions

- **Code** : licence MIT, voir [LICENSE](LICENSE).
- **Données du wiki** : écrites par les contributeurs du wiki Warframe, sous licence [CC BY-NC-SA 3.0](https://creativecommons.org/licenses/by-nc-sa/3.0/). L'archive n'en contient aucune ; l'application les télécharge à la demande. Le dépôt en contient des copies figées pour les tests, dans `tests/fixtures/`, qui restent sous cette licence ; provenance et date : [tests/fixtures/README.md](tests/fixtures/README.md).
- **Polices** : Roboto et Quantico, licence SIL OFL 1.1 (`ui/fonts/`).
- **Bibliothèques embarquées** : [NOTICES.md](NOTICES.md), régénéré par `pwsh -NoProfile -File tools/New-Notices.ps1`.

Warframe est une marque de Digital Extremes Ltd. Plop-Warframe est un projet de fan non commercial, ni affilié ni approuvé par Digital Extremes ou par le wiki.
