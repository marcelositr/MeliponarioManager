PRAGMA foreign_keys = ON;

CREATE TABLE meliponaries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    location TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE species (
    id TEXT PRIMARY KEY,
    common_name TEXT NOT NULL,
    scientific_name TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE hive_boxes (
    id TEXT PRIMARY KEY,
    meliponary_id TEXT NOT NULL,
    code TEXT NOT NULL,
    model TEXT,
    location TEXT,
    installed_at TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (meliponary_id) REFERENCES meliponaries(id),
    UNIQUE (meliponary_id, code)
);

CREATE TABLE colonies (
    id TEXT PRIMARY KEY,
    meliponary_id TEXT NOT NULL,
    species_id TEXT NOT NULL,
    code TEXT NOT NULL,
    origin_type TEXT,
    origin_notes TEXT,
    installed_at TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (meliponary_id) REFERENCES meliponaries(id),
    FOREIGN KEY (species_id) REFERENCES species(id),
    UNIQUE (meliponary_id, code)
);

CREATE TABLE colony_box_placements (
    id TEXT PRIMARY KEY,
    colony_id TEXT NOT NULL,
    hive_box_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    reason TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (colony_id) REFERENCES colonies(id),
    FOREIGN KEY (hive_box_id) REFERENCES hive_boxes(id)
);

CREATE UNIQUE INDEX idx_one_open_placement_per_colony
ON colony_box_placements(colony_id)
WHERE ended_at IS NULL;

CREATE TABLE inspections (
    id TEXT PRIMARY KEY,
    colony_id TEXT NOT NULL,
    inspected_at TEXT NOT NULL,
    strength TEXT CHECK (strength IN ('forte', 'media', 'fraca') OR strength IS NULL),
    laying TEXT,
    brood TEXT,
    food_reserve TEXT,
    queen_presence INTEGER CHECK (queen_presence IN (0, 1) OR queen_presence IS NULL),
    pests TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (colony_id) REFERENCES colonies(id)
);

CREATE TABLE colony_events (
    id TEXT PRIMARY KEY,
    colony_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    description TEXT,
    payload_json TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (colony_id) REFERENCES colonies(id)
);

CREATE INDEX idx_colonies_meliponary ON colonies(meliponary_id);
CREATE INDEX idx_colonies_species ON colonies(species_id);
CREATE INDEX idx_inspections_colony_date ON inspections(colony_id, inspected_at DESC);
CREATE INDEX idx_events_colony_date ON colony_events(colony_id, occurred_at DESC);
