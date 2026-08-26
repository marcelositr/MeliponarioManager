CREATE TABLE inspections (
    id TEXT PRIMARY KEY NOT NULL,
    colony_id TEXT NOT NULL,
    box_id TEXT,
    inspected_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    strength TEXT NOT NULL DEFAULT 'unknown'
        CHECK (strength IN ('strong', 'medium', 'weak', 'unknown')),
    queen_present INTEGER
        CHECK (queen_present IS NULL OR queen_present IN (0, 1)),
    laying_status TEXT,
    food_reserves TEXT,
    brood_status TEXT,
    pests_notes TEXT,
    observations TEXT,
    actions_taken TEXT,
    next_inspection_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    FOREIGN KEY (box_id) REFERENCES boxes(id) ON DELETE RESTRICT
);

CREATE INDEX idx_inspections_colony_date
    ON inspections (colony_id, inspected_at DESC);

CREATE INDEX idx_inspections_next_date
    ON inspections (next_inspection_at)
    WHERE next_inspection_at IS NOT NULL;
