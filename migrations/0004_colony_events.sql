CREATE TABLE colony_events (
    id TEXT PRIMARY KEY,
    colony_id TEXT NOT NULL,
    box_id TEXT,
    event_type TEXT NOT NULL CHECK (
        event_type IN (
            'swarming',
            'abandonment',
            'queen_loss',
            'attack',
            'pest',
            'recovery',
            'maintenance',
            'observation',
            'other'
        )
    ),
    occurred_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    title TEXT,
    details TEXT,
    severity TEXT NOT NULL DEFAULT 'info' CHECK (
        severity IN ('info', 'attention', 'critical')
    ),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    FOREIGN KEY (box_id) REFERENCES boxes(id) ON DELETE SET NULL
);

CREATE INDEX idx_colony_events_colony_date
    ON colony_events (colony_id, occurred_at DESC);

CREATE INDEX idx_colony_events_type
    ON colony_events (event_type);
