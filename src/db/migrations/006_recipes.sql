CREATE TABLE recipe (
    result TEXT PRIMARY KEY,
    json TEXT NOT NULL
);
CREATE TABLE local_recipe (
    result TEXT PRIMARY KEY,
    json TEXT NOT NULL
);
CREATE TABLE character_profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    max_enemy_level INTEGER CHECK (max_enemy_level > 0)
);
INSERT INTO character_profile(id, max_enemy_level) VALUES (1, NULL);
