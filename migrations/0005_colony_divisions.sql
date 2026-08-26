PRAGMA foreign_keys = ON;

CREATE TABLE colony_divisions (
    id TEXT PRIMARY KEY NOT NULL,
    parent_colony_id TEXT NOT NULL,
    daughter_colony_id TEXT,
    source_box_id TEXT,
    performed_at TEXT NOT NULL,
    result TEXT NOT NULL DEFAULT 'successful'
        CHECK (result IN ('successful', 'partial', 'failed')),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    FOREIGN KEY (daughter_colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    FOREIGN KEY (source_box_id) REFERENCES boxes(id) ON DELETE RESTRICT,
    UNIQUE (daughter_colony_id),
    CHECK (daughter_colony_id IS NULL OR daughter_colony_id <> parent_colony_id),
    CHECK (
        (result = 'failed' AND daughter_colony_id IS NULL)
        OR
        (result IN ('successful', 'partial') AND daughter_colony_id IS NOT NULL)
    )
);

CREATE INDEX ix_divisions_parent
    ON colony_divisions(parent_colony_id, performed_at DESC);

CREATE INDEX ix_divisions_daughter
    ON colony_divisions(daughter_colony_id, performed_at DESC);
