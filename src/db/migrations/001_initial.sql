-- Schema v1. Reference: docs/05-DATA-SCHEMAS.md.

CREATE TABLE planet (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE mission_type (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    is_endless INTEGER NOT NULL DEFAULT 0 CHECK (is_endless IN (0, 1))
);

CREATE TABLE node (
    id              INTEGER PRIMARY KEY,
    key             TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    planet_id       INTEGER NOT NULL REFERENCES planet (id),
    mission_type_id INTEGER NOT NULL REFERENCES mission_type (id),
    faction         TEXT,
    min_level       INTEGER,
    max_level       INTEGER,
    is_dark_sector  INTEGER NOT NULL DEFAULT 0 CHECK (is_dark_sector IN (0, 1)),
    is_special      INTEGER NOT NULL DEFAULT 0 CHECK (is_special IN (0, 1))
);
CREATE INDEX node_mission_type ON node (mission_type_id);

CREATE TABLE drop_table (
    id            INTEGER PRIMARY KEY,
    alias         TEXT NOT NULL UNIQUE,
    name          TEXT NOT NULL,
    table_type    TEXT NOT NULL,
    tier          TEXT,
    rotation_kind TEXT NOT NULL CHECK (rotation_kind IN ('rotation', 'cache', 'stage', 'none'))
);

CREATE TABLE node_drop_table (
    node_id       INTEGER NOT NULL REFERENCES node (id),
    drop_table_id INTEGER NOT NULL REFERENCES drop_table (id),
    role          TEXT NOT NULL CHECK (role IN ('main', 'cache', 'extra')),
    PRIMARY KEY (node_id, drop_table_id, role)
);
CREATE INDEX node_drop_table_table ON node_drop_table (drop_table_id);

CREATE TABLE item (
    id       INTEGER PRIMARY KEY,
    name     TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL
);

CREATE TABLE "drop" (
    id            INTEGER PRIMARY KEY,
    drop_table_id INTEGER NOT NULL REFERENCES drop_table (id),
    rotation      TEXT NOT NULL CHECK (rotation IN ('A', 'B', 'C', 'NONE')),
    item_id       INTEGER NOT NULL REFERENCES item (id),
    chance        REAL NOT NULL CHECK (chance >= 0 AND chance <= 1),
    rarity        TEXT NOT NULL,
    quantity      INTEGER NOT NULL CHECK (quantity >= 1)
);
CREATE INDEX drop_item ON "drop" (item_id);
CREATE INDEX drop_table_rotation ON "drop" (drop_table_id, rotation);

CREATE TABLE import_run (
    id              INTEGER PRIMARY KEY,
    started_at      INTEGER NOT NULL,
    finished_at     INTEGER,
    source          TEXT NOT NULL,
    source_revision TEXT,
    status          TEXT NOT NULL CHECK (status IN ('running', 'success', 'failed')),
    message         TEXT
);
