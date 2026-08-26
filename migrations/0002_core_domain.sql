PRAGMA foreign_keys = ON;

CREATE TABLE meliponaries (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    responsible_name TEXT,
    location TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE species (
    id TEXT PRIMARY KEY NOT NULL,
    common_name TEXT NOT NULL CHECK (length(trim(common_name)) > 0),
    scientific_name TEXT,
    genus TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE boxes (
    id TEXT PRIMARY KEY NOT NULL,
    meliponary_id TEXT NOT NULL,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    model TEXT,
    material TEXT,
    location_note TEXT,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'maintenance', 'retired')),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (meliponary_id) REFERENCES meliponaries(id) ON DELETE RESTRICT,
    UNIQUE (meliponary_id, code)
);

CREATE TABLE colonies (
    id TEXT PRIMARY KEY NOT NULL,
    meliponary_id TEXT NOT NULL,
    species_id TEXT NOT NULL,
    code TEXT NOT NULL CHECK (length(trim(code)) > 0),
    origin_type TEXT NOT NULL DEFAULT 'historical'
        CHECK (origin_type IN ('acquisition', 'multiplication', 'transfer', 'rescue', 'authorized_capture', 'historical', 'other')),
    origin_notes TEXT,
    installed_at TEXT,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'weak', 'recovering', 'transferred', 'lost', 'inactive')),
    mother_colony_id TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (meliponary_id) REFERENCES meliponaries(id) ON DELETE RESTRICT,
    FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE RESTRICT,
    FOREIGN KEY (mother_colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    UNIQUE (meliponary_id, code),
    CHECK (mother_colony_id IS NULL OR mother_colony_id <> id)
);

CREATE TABLE colony_box_occupancies (
    id TEXT PRIMARY KEY NOT NULL,
    colony_id TEXT NOT NULL,
    box_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    reason TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    FOREIGN KEY (box_id) REFERENCES boxes(id) ON DELETE RESTRICT,
    CHECK (ended_at IS NULL OR ended_at >= started_at)
);

CREATE UNIQUE INDEX ux_colony_active_box
    ON colony_box_occupancies(colony_id)
    WHERE ended_at IS NULL;

CREATE UNIQUE INDEX ux_box_active_colony
    ON colony_box_occupancies(box_id)
    WHERE ended_at IS NULL;

CREATE INDEX ix_colonies_meliponary ON colonies(meliponary_id);
CREATE INDEX ix_colonies_species ON colonies(species_id);
CREATE INDEX ix_boxes_meliponary ON boxes(meliponary_id);
CREATE INDEX ix_occupancies_colony_history ON colony_box_occupancies(colony_id, started_at DESC);
CREATE INDEX ix_occupancies_box_history ON colony_box_occupancies(box_id, started_at DESC);
