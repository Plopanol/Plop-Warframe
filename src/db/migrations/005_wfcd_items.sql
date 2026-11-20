-- Schema v5: every @wfcd/items item, with its raw JSON. Reference: docs/05-DATA-SCHEMAS.md.

CREATE TABLE wfcd_item (
    id          INTEGER PRIMARY KEY,
    unique_name TEXT NOT NULL UNIQUE,
    category    TEXT NOT NULL,
    name        TEXT NOT NULL,
    name_fr     TEXT,
    json        TEXT NOT NULL,
    json_fr     TEXT
);
CREATE INDEX wfcd_item_category ON wfcd_item (category);
