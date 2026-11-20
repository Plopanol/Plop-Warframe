-- Schema v3: contents of the Void relics (DE drop tables). Reference: docs/05-DATA-SCHEMAS.md.

CREATE TABLE relic (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tier TEXT NOT NULL
);

CREATE TABLE relic_reward (
    id         INTEGER PRIMARY KEY,
    relic_id   INTEGER NOT NULL REFERENCES relic (id),
    refinement TEXT NOT NULL CHECK (refinement IN ('Intact', 'Exceptional', 'Flawless', 'Radiant')),
    item_name  TEXT NOT NULL,
    chance     REAL NOT NULL CHECK (chance >= 0 AND chance <= 1),
    rarity     TEXT NOT NULL
);
CREATE INDEX relic_reward_item ON relic_reward (item_name);
CREATE INDEX relic_reward_relic ON relic_reward (relic_id);
