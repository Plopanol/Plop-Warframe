# Prompt — Application « Warframe Drop Finder » (Rust + Slint + SQLite)

## Rôle

Tu es un développeur Rust senior, à l'aise avec Slint, SQLite (rusqlite) et le scraping de données web. Tu vas concevoir et implémenter de bout en bout une application desktop, de manière incrémentale, en livrant du code compilable à chaque étape.

## Objectif de l'application

Une application desktop (Windows et Linux) qui :

1. **Importe** les tables de drop de Warframe depuis le wiki officiel (https://wiki.warframe.com/w/Drop_Tables) et les enregistre dans une base **SQLite** locale.
2. Propose un **écran de recherche** qui, pour un **type de mission** donné (Survie, Défense, Excavation, Interception, Capture, Disruption…) et un ou plusieurs **objets sélectionnés**, affiche les **lieux de mission (planète + nœud)** qui ont le **taux de drop le plus élevé** pour ces objets, triés par taux décroissant.

## Stack technique imposée

- **Rust** stable (édition 2021 ou plus récente), un seul crate binaire.
- **Slint** 1.x pour l'UI (fichiers `.slint` compilés via `slint-build` dans `build.rs`).
- **rusqlite** avec la feature `bundled` (pas de dépendance système à SQLite).
- **reqwest** pour le HTTP (client bloquant exécuté dans un thread dédié, ou tokio si tu le justifies).
- **serde / serde_json**, **anyhow** (binaire) + **thiserror** (erreurs métier), **directories** pour le chemin de la base, **log + env_logger** (ou tracing).
- Pour le parsing : voir la section « Source des données » (probablement **mlua** et/ou **scraper**).
- Utilise les dernières versions stables des crates (vérifie sur crates.io) et justifie tout crate supplémentaire.

## Source des données — à investiguer en premier

La page `Drop_Tables` du wiki est générée à partir de **modules Lua** MediaWiki. Les données brutes se trouvent a priori dans :

- `Module:DropTables/data` → les tables de drop (alias de table, rotations A/B/C, objets, chances, rareté, quantité).
- `Module:Missions/data` → les nœuds de la carte (planète, nom du nœud, type de mission, faction, niveaux) et l'alias de table de drop utilisé par chaque nœud.

Le contenu brut d'un module s'obtient avec `?action=raw`, par ex. :
`https://wiki.warframe.com/w/Module:DropTables/data?action=raw`

**Étape 0 obligatoire, avant d'écrire le parseur :**
1. Télécharge ces deux modules (et la page HTML `Drop_Tables` en secours), sauvegarde-les dans `tests/fixtures/`.
2. Analyse leur structure réelle (clés, imbrication, format des chances : pourcentage ou fraction, présence de `require(...)` vers d'autres modules).
3. Résume-moi la structure trouvée et le plan de mapping vers le schéma SQL **avant** d'implémenter l'import.

**Stratégie de parsing recommandée :**
- **Option A (privilégiée)** : exécuter le module Lua dans un interpréteur embarqué (**mlua**, features `lua54` + `vendored` + `serialize`), récupérer la table retournée et la désérialiser avec serde. Si le module appelle `require('Module:…')`, fournis une fonction `require` personnalisée qui télécharge le module correspondant en `?action=raw` (avec cache), ou des stubs si ce n'est pas nécessaire aux données.
- **Option B (secours)** : parser le HTML rendu de `Drop_Tables` (crate `scraper`) si les modules Lua sont inexploitables.
- Garde le parseur derrière un trait (`DropSource`) pour pouvoir changer de source plus tard (ex. tables officielles de DE sur https://www.warframe.com/droptables).

**Bonnes pratiques réseau :** User-Agent explicite identifiant l'application, requêtes séquentielles avec un petit délai, timeout, cache des fichiers bruts sur disque avec leur date de téléchargement, possibilité d'importer depuis le cache sans réseau.

## Modèle de données SQLite

Propose et implémente un schéma normalisé, au minimum :

- `planet(id, name)`
- `mission_type(id, name)`
- `node(id, name, planet_id, mission_type_id, faction, min_level, max_level, is_dark_sector, …)`
- `drop_table(id, alias, name)` — table de récompenses (une table peut être partagée par plusieurs nœuds)
- `node_drop_table(node_id, drop_table_id)`
- `item(id, name, category)` — catégorie : relique, mod, ressource, plan, crédits, etc. si l'info est disponible
- `drop(id, drop_table_id, rotation, item_id, chance, rarity, quantity)` — `rotation` ∈ {`A`,`B`,`C`, `NONE`} ; `chance` stockée en **REAL entre 0 et 1**
- `import_run(id, started_at, finished_at, source, source_revision, status, message)`

Contraintes : clés étrangères activées, index sur `item.name`, `drop.item_id`, `node.mission_type_id`. Migrations versionnées via `PRAGMA user_version`. L'import est **idempotent** : il s'exécute dans **une seule transaction** qui vide puis recharge les tables de données (l'historique `import_run` est conservé).

Les missions sans nœud fixe (Sorties, Arbitrages, Cauchemar, primes, missions à clé…) : importe-les si le format le permet, avec un champ indiquant leur nature, et ajoute un filtre pour les inclure ou non dans la recherche.

## Logique de recherche

Entrées : un type de mission, une liste d'objets (1..n), un filtre optionnel de rotation (A/B/C/toutes), un filtre « inclure les missions spéciales ».

Deux modes d'affichage (sélecteur dans l'UI) :

1. **Par objet** : pour chaque objet sélectionné, la liste des nœuds du type choisi qui le donnent, triés par chance décroissante. Colonnes : Objet, Planète, Nœud, Rotation, Chance (%), Rareté, Faction, Niveaux.
2. **Combiné** : pour chaque nœud et chaque rotation, la **somme** des chances des objets sélectionnés (les entrées d'une même rotation s'excluent mutuellement : un tirage donne un seul objet, la somme est donc la probabilité d'obtenir l'un des objets voulus). Tri décroissant sur cette somme. Colonnes : Planète, Nœud, Rotation, Chance cumulée (%), détail des objets couverts.

Colonne optionnelle utile : **chance par cycle AABC** pour les missions infinies = `(2·A + B + C) / 4`.

La requête doit être faite en SQL (pas de filtrage en mémoire sur toute la base) et répondre en < 100 ms sur la base complète. Fournis la requête et un test qui la valide.

## Interface Slint

Langue de l'interface : **français**. Deux onglets :

**Onglet « Données »**
- Date et statut du dernier import, nombre de nœuds / objets / drops en base.
- Bouton « Mettre à jour depuis le wiki » et bouton « Réimporter depuis le cache ».
- Barre de progression + zone de log. L'import tourne dans un **thread d'arrière-plan** ; l'UI est mise à jour via `slint::Weak` + `slint::invoke_from_event_loop`. L'UI ne doit jamais se figer.

**Onglet « Recherche »**
- ComboBox du type de mission (alimentée depuis la base).
- Champ de recherche d'objet avec autocomplétion (filtre « contient », insensible à la casse et aux accents), résultats cliquables pour ajouter l'objet à la sélection ; la sélection s'affiche sous forme de puces supprimables.
- Filtres rotation et missions spéciales, sélecteur de mode (Par objet / Combiné).
- Résultats dans un `StandardTableView` triable par colonne.
- Message clair si la base est vide (« Lancez un import depuis l'onglet Données »).

## Architecture attendue

```
Cargo.toml
build.rs                 # slint_build::compile("ui/app.slint")
ui/
  app.slint              # fenêtre + onglets
  data_tab.slint
  search_tab.slint
src/
  main.rs                # bootstrap, câblage callbacks Slint
  source/                # trait DropSource, impl wiki (lua / html), cache HTTP
  model.rs               # structs métier (Node, DropEntry, …)
  db/
    mod.rs               # ouverture, migrations
    import.rs            # écriture transactionnelle
    query.rs             # requêtes de recherche
  ui_bridge.rs           # conversion modèle ↔ modèles Slint (VecModel, StandardListViewItem)
tests/
  fixtures/              # modules Lua / HTML sauvegardés
  parse_tests.rs
  query_tests.rs
```

La base est stockée dans le dossier de données utilisateur (`directories::ProjectDirs`), son chemin est affiché dans l'onglet Données.

## Qualité

- Pas de `unwrap()`/`expect()` hors tests et initialisation triviale ; erreurs remontées à l'UI avec un message lisible.
- `cargo clippy -- -D warnings` et `cargo fmt` propres.
- Tests unitaires du parseur sur les fixtures (nombre de nœuds, quelques drops connus vérifiés), tests de la requête de recherche sur une petite base en mémoire.
- README court : compilation, lancement, où se trouve la base, comment mettre à jour.

## Plan de livraison (une étape à la fois, en t'arrêtant pour validation)

1. Étape 0 : téléchargement des sources + analyse de leur structure → **compte rendu et mapping proposés**.
2. Parseur + tests sur fixtures.
3. Schéma SQLite, migrations, import transactionnel + CLI minimal (`--import`) pour vérifier.
4. Requêtes de recherche + tests.
5. UI Slint : onglet Données (import en arrière-plan).
6. UI Slint : onglet Recherche.
7. Finitions : gestion d'erreurs, README, clippy.

## Critères d'acceptation

- Un import complet depuis le wiki aboutit et remplit la base sans doublons ; un second import donne le même résultat.
- Exemple : en choisissant « Survie » et une relique donnée, l'application liste les nœuds de Survie qui la donnent, avec la rotation et le pourcentage, la meilleure chance en premier, et les valeurs correspondent à celles affichées sur le wiki.
- Avec plusieurs objets en mode Combiné, la chance cumulée par nœud/rotation est correcte.
- L'application compile et fonctionne sous Windows et Linux.

Si une information te manque ou si la structure réelle des données contredit ce prompt, pose-moi la question au lieu de supposer.
