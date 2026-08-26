PRAGMA foreign_keys = ON;

CREATE TABLE colony_movements (
    id TEXT PRIMARY KEY NOT NULL,
    colony_id TEXT NOT NULL,
    movement_type TEXT NOT NULL
        CHECK (movement_type IN ('internal_transfer', 'external_transfer', 'transport')),
    moved_at TEXT NOT NULL,
    from_meliponary_id TEXT NOT NULL,
    to_meliponary_id TEXT,
    from_box_id TEXT,
    to_box_id TEXT,
    destination TEXT,
    document_reference TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    FOREIGN KEY (from_meliponary_id) REFERENCES meliponaries(id) ON DELETE RESTRICT,
    FOREIGN KEY (to_meliponary_id) REFERENCES meliponaries(id) ON DELETE RESTRICT,
    FOREIGN KEY (from_box_id) REFERENCES boxes(id) ON DELETE RESTRICT,
    FOREIGN KEY (to_box_id) REFERENCES boxes(id) ON DELETE RESTRICT,
    CHECK (to_meliponary_id IS NULL OR to_meliponary_id <> from_meliponary_id)
);

CREATE INDEX ix_movements_colony_history
    ON colony_movements(colony_id, moved_at DESC);

CREATE INDEX ix_movements_from_meliponary
    ON colony_movements(from_meliponary_id, moved_at DESC);

CREATE INDEX ix_movements_to_meliponary
    ON colony_movements(to_meliponary_id, moved_at DESC);
