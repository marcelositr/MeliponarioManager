PRAGMA foreign_keys = ON;

CREATE TABLE box_maintenance_records (
    id TEXT PRIMARY KEY NOT NULL,
    box_id TEXT NOT NULL,
    colony_id TEXT,
    maintained_at TEXT NOT NULL,
    maintenance_type TEXT NOT NULL
        CHECK (maintenance_type IN (
            'cleaning',
            'repair',
            'painting',
            'waterproofing',
            'roof',
            'entrance',
            'internal_structure',
            'inspection',
            'other'
        )),
    description TEXT,
    performed_by TEXT,
    cost REAL CHECK (cost IS NULL OR cost >= 0),
    next_maintenance_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (box_id) REFERENCES boxes(id) ON DELETE RESTRICT,
    FOREIGN KEY (colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    CHECK (next_maintenance_at IS NULL OR next_maintenance_at >= maintained_at)
);

CREATE INDEX ix_box_maintenance_box_history
    ON box_maintenance_records(box_id, maintained_at DESC);

CREATE INDEX ix_box_maintenance_colony_history
    ON box_maintenance_records(colony_id, maintained_at DESC)
    WHERE colony_id IS NOT NULL;

CREATE INDEX ix_box_maintenance_next
    ON box_maintenance_records(next_maintenance_at)
    WHERE next_maintenance_at IS NOT NULL;
