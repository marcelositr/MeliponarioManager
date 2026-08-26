CREATE TABLE feedings (
    id TEXT PRIMARY KEY NOT NULL,
    colony_id TEXT NOT NULL,
    box_id TEXT,
    fed_at TEXT NOT NULL,
    food_type TEXT NOT NULL CHECK (length(trim(food_type)) > 0),
    quantity REAL CHECK (quantity IS NULL OR quantity > 0),
    unit TEXT,
    response_notes TEXT,
    notes TEXT,
    next_feeding_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    FOREIGN KEY (box_id) REFERENCES boxes(id) ON DELETE RESTRICT,
    CHECK (
        (quantity IS NULL AND unit IS NULL)
        OR
        (quantity IS NOT NULL AND unit IS NOT NULL AND length(trim(unit)) > 0)
    )
);

CREATE INDEX ix_feedings_colony_history
    ON feedings(colony_id, fed_at DESC);

CREATE INDEX ix_feedings_next_due
    ON feedings(next_feeding_at)
    WHERE next_feeding_at IS NOT NULL;
