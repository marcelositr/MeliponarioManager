PRAGMA foreign_keys = ON;

CREATE TABLE box_state_records (
    id TEXT PRIMARY KEY NOT NULL,
    box_id TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    previous_status TEXT NOT NULL
        CHECK (previous_status IN ('active', 'maintenance', 'retired')),
    new_status TEXT NOT NULL
        CHECK (new_status IN ('active', 'maintenance', 'retired')),
    reason TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (box_id) REFERENCES boxes(id) ON DELETE RESTRICT,
    CHECK (previous_status <> new_status)
);

CREATE INDEX ix_box_state_history
    ON box_state_records(box_id, occurred_at DESC, created_at DESC);

CREATE TRIGGER trg_occupancy_requires_active_box
BEFORE INSERT ON colony_box_occupancies
FOR EACH ROW
WHEN COALESCE((SELECT status FROM boxes WHERE id = NEW.box_id), '') <> 'active'
BEGIN
    SELECT RAISE(ABORT, 'Caixa indisponível para nova ocupação.');
END;

CREATE TRIGGER trg_nonactive_box_requires_no_active_occupancy
BEFORE UPDATE OF status ON boxes
FOR EACH ROW
WHEN NEW.status IN ('maintenance', 'retired')
 AND OLD.status <> NEW.status
 AND EXISTS (
     SELECT 1
     FROM colony_box_occupancies
     WHERE box_id = NEW.id AND ended_at IS NULL
 )
BEGIN
    SELECT RAISE(ABORT, 'Caixa ocupada não pode sair do estado ativo.');
END;
