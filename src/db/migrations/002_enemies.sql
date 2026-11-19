-- Schema v2: enemy drops and locations. Reference: docs/05-DATA-SCHEMAS.md.

ALTER TABLE node ADD COLUMN tileset TEXT;

CREATE TABLE node_faction (
    node_id INTEGER NOT NULL REFERENCES node (id),
    faction TEXT NOT NULL,
    PRIMARY KEY (node_id, faction)
);
CREATE INDEX node_faction_faction ON node_faction (faction);

CREATE TABLE enemy (
    id      INTEGER PRIMARY KEY,
    key     TEXT NOT NULL UNIQUE,
    name    TEXT NOT NULL,
    faction TEXT
);

CREATE TABLE enemy_location (
    enemy_id INTEGER NOT NULL REFERENCES enemy (id),
    kind     TEXT NOT NULL CHECK (kind IN ('planet', 'tileset')),
    value    TEXT NOT NULL,
    PRIMARY KEY (enemy_id, kind, value)
);

CREATE TABLE enemy_drop (
    id       INTEGER PRIMARY KEY,
    enemy_id INTEGER NOT NULL REFERENCES enemy (id),
    item_id  INTEGER NOT NULL REFERENCES item (id),
    chance   REAL NOT NULL CHECK (chance >= 0 AND chance <= 1),
    rarity   TEXT NOT NULL
);
CREATE INDEX enemy_drop_item ON enemy_drop (item_id);
CREATE INDEX enemy_drop_enemy ON enemy_drop (enemy_id);

CREATE TABLE region_resource (
    planet_id INTEGER NOT NULL REFERENCES planet (id),
    item_id   INTEGER NOT NULL REFERENCES item (id),
    PRIMARY KEY (planet_id, item_id)
);
