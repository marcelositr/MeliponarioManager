PRAGMA foreign_keys = ON;

CREATE TABLE movement_documents (
    id TEXT PRIMARY KEY NOT NULL,
    movement_id TEXT NOT NULL,
    document_type TEXT NOT NULL
        CHECK (document_type IN (
            'gta',
            'authorization',
            'invoice',
            'receipt',
            'declaration',
            'protocol',
            'certificate',
            'other'
        )),
    reference_number TEXT NOT NULL
        CHECK (LENGTH(TRIM(reference_number)) > 0),
    source_system TEXT,
    issuer TEXT,
    issued_at TEXT,
    valid_until TEXT,
    file_path TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (movement_id) REFERENCES colony_movements(id) ON DELETE RESTRICT,
    CHECK (valid_until IS NULL OR issued_at IS NULL OR valid_until >= issued_at)
);

CREATE UNIQUE INDEX ux_movement_document_reference
    ON movement_documents(movement_id, document_type, reference_number);

CREATE INDEX ix_movement_documents_movement
    ON movement_documents(movement_id, issued_at DESC, created_at DESC);

CREATE INDEX ix_movement_documents_type_reference
    ON movement_documents(document_type, reference_number);

-- Migra referências simples gravadas antes da normalização documental.
INSERT INTO movement_documents (
    id,
    movement_id,
    document_type,
    reference_number,
    notes
)
SELECT
    'legacy-' || id,
    id,
    'other',
    TRIM(document_reference),
    'Importado do campo legado document_reference.'
FROM colony_movements
WHERE document_reference IS NOT NULL
  AND TRIM(document_reference) <> '';

UPDATE colony_movements
SET document_reference = NULL
WHERE document_reference IS NOT NULL
  AND TRIM(document_reference) <> '';

-- Mantém compatibilidade com clientes antigos que ainda enviem
-- document_reference ao criar uma movimentação. O valor é normalizado
-- imediatamente e o campo legado volta a ficar vazio.
CREATE TRIGGER trg_bridge_movement_document_reference
AFTER INSERT ON colony_movements
WHEN NEW.document_reference IS NOT NULL
 AND TRIM(NEW.document_reference) <> ''
BEGIN
    INSERT INTO movement_documents (
        id,
        movement_id,
        document_type,
        reference_number,
        notes
    ) VALUES (
        'legacy-' || NEW.id,
        NEW.id,
        'other',
        TRIM(NEW.document_reference),
        'Criado a partir do campo legado document_reference.'
    );

    UPDATE colony_movements
    SET document_reference = NULL
    WHERE id = NEW.id;
END;
