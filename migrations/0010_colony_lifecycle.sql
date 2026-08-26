PRAGMA foreign_keys = ON;

CREATE TABLE colony_lifecycle_records (
    id TEXT PRIMARY KEY NOT NULL,
    colony_id TEXT NOT NULL,
    box_id TEXT,
    action TEXT NOT NULL
        CHECK (action IN ('loss', 'deactivate', 'reactivate')),
    occurred_at TEXT NOT NULL,
    previous_status TEXT NOT NULL
        CHECK (previous_status IN ('active', 'weak', 'recovering', 'transferred', 'lost', 'inactive')),
    new_status TEXT NOT NULL
        CHECK (new_status IN ('active', 'weak', 'recovering', 'transferred', 'lost', 'inactive')),
    reason TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    FOREIGN KEY (box_id) REFERENCES boxes(id) ON DELETE RESTRICT,
    CHECK (previous_status <> new_status)
);

CREATE INDEX ix_colony_lifecycle_history
    ON colony_lifecycle_records(colony_id, occurred_at DESC);

CREATE TRIGGER trg_occupancy_requires_manageable_colony
BEFORE INSERT ON colony_box_occupancies
FOR EACH ROW
WHEN COALESCE((SELECT status FROM colonies WHERE id = NEW.colony_id), '')
     NOT IN ('active', 'weak', 'recovering')
BEGIN
    SELECT RAISE(ABORT, 'Colônia indisponível para ocupação de caixa.');
END;

CREATE TRIGGER trg_terminal_colony_closes_active_box
AFTER UPDATE OF status ON colonies
FOR EACH ROW
WHEN NEW.status IN ('lost', 'inactive', 'transferred')
 AND OLD.status <> NEW.status
BEGIN
    UPDATE colony_box_occupancies
    SET ended_at = CURRENT_TIMESTAMP
    WHERE colony_id = NEW.id AND ended_at IS NULL;
END;
