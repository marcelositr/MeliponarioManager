PRAGMA foreign_keys = ON;

CREATE TABLE inspection_photos (
    id TEXT PRIMARY KEY NOT NULL,
    inspection_id TEXT NOT NULL,
    relative_path TEXT NOT NULL UNIQUE,
    original_name TEXT NOT NULL,
    mime_type TEXT NOT NULL
        CHECK (mime_type IN ('image/jpeg', 'image/png', 'image/webp')),
    byte_size INTEGER NOT NULL CHECK (byte_size >= 0),
    captured_at TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (inspection_id) REFERENCES inspections(id) ON DELETE CASCADE
);

CREATE INDEX ix_inspection_photos_inspection
    ON inspection_photos(inspection_id, captured_at DESC);

CREATE INDEX ix_inspection_photos_captured_at
    ON inspection_photos(captured_at DESC);
