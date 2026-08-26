PRAGMA foreign_keys = ON;

CREATE TABLE production_records (
    id TEXT PRIMARY KEY NOT NULL,
    colony_id TEXT NOT NULL,
    box_id TEXT,
    harvested_at TEXT NOT NULL,
    product_type TEXT NOT NULL
        CHECK (product_type IN ('honey', 'pollen', 'propolis', 'wax', 'cerumen', 'other')),
    quantity REAL NOT NULL CHECK (quantity > 0),
    unit TEXT NOT NULL CHECK (length(trim(unit)) > 0),
    purpose TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    FOREIGN KEY (box_id) REFERENCES boxes(id) ON DELETE RESTRICT
);

CREATE INDEX ix_production_colony_history
    ON production_records(colony_id, harvested_at DESC);

CREATE INDEX ix_production_product_type
    ON production_records(product_type, harvested_at DESC);
