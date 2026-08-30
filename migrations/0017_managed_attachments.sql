PRAGMA foreign_keys = ON;

CREATE TABLE managed_attachments (
    id TEXT PRIMARY KEY NOT NULL,
    meliponary_id TEXT NOT NULL,
    original_name TEXT NOT NULL,
    relative_path TEXT NOT NULL UNIQUE,
    extension TEXT,
    mime_type TEXT,
    byte_size INTEGER NOT NULL CHECK (byte_size >= 0),
    description TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (meliponary_id) REFERENCES meliponaries(id) ON DELETE RESTRICT
);

CREATE INDEX ix_managed_attachments_meliponary
    ON managed_attachments(meliponary_id, created_at DESC);
