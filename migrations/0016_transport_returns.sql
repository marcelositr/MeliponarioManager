PRAGMA foreign_keys = ON;

CREATE TABLE transport_returns (
    id TEXT PRIMARY KEY NOT NULL,
    movement_id TEXT NOT NULL,
    returned_at TEXT NOT NULL,
    notes TEXT,
    reversed_at TEXT,
    reversal_reason TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (movement_id) REFERENCES colony_movements(id) ON DELETE RESTRICT,
    CHECK (reversed_at IS NOT NULL OR reversal_reason IS NULL)
);

CREATE UNIQUE INDEX ux_transport_returns_active
    ON transport_returns(movement_id)
    WHERE reversed_at IS NULL;

CREATE INDEX ix_transport_returns_movement_history
    ON transport_returns(movement_id, returned_at DESC, created_at DESC);

CREATE TRIGGER validate_transport_return_insert
BEFORE INSERT ON transport_returns
BEGIN
    SELECT CASE
        WHEN NOT EXISTS (
            SELECT 1
            FROM colony_movements m
            WHERE m.id = NEW.movement_id
              AND m.movement_type = 'transport'
              AND m.voided_at IS NULL
              AND m.reversed_at IS NULL
        ) THEN RAISE(ABORT, 'Retorno exige um transporte temporário válido.')
    END;

    SELECT CASE
        WHEN NEW.returned_at < (
            SELECT m.moved_at FROM colony_movements m WHERE m.id = NEW.movement_id
        ) THEN RAISE(ABORT, 'O retorno não pode ser anterior ao início do transporte.')
    END;
END;

CREATE TRIGGER prevent_parallel_open_transport
BEFORE INSERT ON colony_movements
WHEN NEW.movement_type = 'transport'
BEGIN
    SELECT CASE
        WHEN EXISTS (
            SELECT 1
            FROM colony_movements m
            WHERE m.colony_id = NEW.colony_id
              AND m.movement_type = 'transport'
              AND m.voided_at IS NULL
              AND m.reversed_at IS NULL
              AND NOT EXISTS (
                  SELECT 1
                  FROM transport_returns r
                  WHERE r.movement_id = m.id
                    AND r.reversed_at IS NULL
              )
        ) THEN RAISE(ABORT, 'A colônia já possui um transporte temporário aberto.')
    END;
END;

CREATE TRIGGER prevent_void_completed_transport
BEFORE UPDATE OF voided_at ON colony_movements
WHEN OLD.movement_type = 'transport'
  AND OLD.voided_at IS NULL
  AND NEW.voided_at IS NOT NULL
  AND EXISTS (
      SELECT 1
      FROM transport_returns r
      WHERE r.movement_id = OLD.id
        AND r.reversed_at IS NULL
  )
BEGIN
    SELECT RAISE(ABORT, 'Transporte concluído precisa ser reaberto antes de ser anulado.');
END;
