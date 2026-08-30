PRAGMA foreign_keys = ON;

CREATE TABLE scheduled_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    meliponary_id TEXT NOT NULL,
    colony_id TEXT,
    box_id TEXT,
    task_type TEXT NOT NULL
        CHECK (task_type IN ('inspection', 'feeding', 'maintenance', 'generic')),
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    description TEXT,
    scheduled_for TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'completed', 'cancelled', 'rescheduled', 'skipped')),
    priority TEXT NOT NULL DEFAULT 'normal'
        CHECK (priority IN ('normal', 'attention', 'critical')),
    source_type TEXT,
    source_id TEXT,
    completed_at TEXT,
    completed_by_type TEXT,
    completed_by_id TEXT,
    cancelled_at TEXT,
    cancellation_reason TEXT,
    skipped_at TEXT,
    skip_reason TEXT,
    rescheduled_from_id TEXT,
    reschedule_reason TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'localtime')),
    FOREIGN KEY (meliponary_id) REFERENCES meliponaries(id) ON DELETE RESTRICT,
    FOREIGN KEY (colony_id) REFERENCES colonies(id) ON DELETE RESTRICT,
    FOREIGN KEY (box_id) REFERENCES boxes(id) ON DELETE RESTRICT,
    FOREIGN KEY (rescheduled_from_id) REFERENCES scheduled_tasks(id) ON DELETE RESTRICT,
    CHECK (source_type IS NULL OR source_id IS NOT NULL),
    CHECK (status <> 'completed' OR completed_at IS NOT NULL),
    CHECK (status <> 'cancelled' OR (cancelled_at IS NOT NULL AND length(trim(cancellation_reason)) > 0)),
    CHECK (status <> 'skipped' OR (skipped_at IS NOT NULL AND length(trim(skip_reason)) > 0))
);

CREATE INDEX ix_scheduled_tasks_status_date
    ON scheduled_tasks(status, scheduled_for);
CREATE INDEX ix_scheduled_tasks_meliponary_date
    ON scheduled_tasks(meliponary_id, status, scheduled_for);
CREATE INDEX ix_scheduled_tasks_colony_date
    ON scheduled_tasks(colony_id, status, scheduled_for)
    WHERE colony_id IS NOT NULL;
CREATE INDEX ix_scheduled_tasks_box_date
    ON scheduled_tasks(box_id, status, scheduled_for)
    WHERE box_id IS NOT NULL;
CREATE INDEX ix_scheduled_tasks_source
    ON scheduled_tasks(source_type, source_id)
    WHERE source_type IS NOT NULL AND source_id IS NOT NULL;
CREATE UNIQUE INDEX ux_scheduled_tasks_pending_source
    ON scheduled_tasks(source_type, source_id)
    WHERE status = 'pending' AND source_type IS NOT NULL AND source_id IS NOT NULL;
CREATE INDEX ix_scheduled_tasks_reschedule_lineage
    ON scheduled_tasks(rescheduled_from_id)
    WHERE rescheduled_from_id IS NOT NULL;

WITH ranked AS (
    SELECT i.id, i.colony_id, i.box_id, i.next_inspection_at,
           c.meliponary_id, c.code AS colony_code,
           ROW_NUMBER() OVER (
               PARTITION BY i.colony_id
               ORDER BY i.inspected_at DESC, i.created_at DESC, i.id DESC
           ) AS rn
    FROM inspections i
    JOIN colonies c ON c.id = i.colony_id
    JOIN meliponaries m ON m.id = c.meliponary_id
    WHERE i.voided_at IS NULL
      AND c.status IN ('active', 'weak', 'recovering')
      AND m.archived_at IS NULL
)
INSERT INTO scheduled_tasks (
    id, meliponary_id, colony_id, box_id, task_type, title,
    scheduled_for, status, priority, source_type, source_id
)
SELECT lower(hex(randomblob(16))), meliponary_id, colony_id, box_id,
       'inspection', 'Inspecionar ' || colony_code,
       next_inspection_at, 'pending', 'normal', 'inspection', id
FROM ranked
WHERE rn = 1 AND next_inspection_at IS NOT NULL;

WITH ranked AS (
    SELECT f.id, f.colony_id, f.box_id, f.next_feeding_at,
           c.meliponary_id, c.code AS colony_code,
           ROW_NUMBER() OVER (
               PARTITION BY f.colony_id
               ORDER BY f.fed_at DESC, f.created_at DESC, f.id DESC
           ) AS rn
    FROM feedings f
    JOIN colonies c ON c.id = f.colony_id
    JOIN meliponaries m ON m.id = c.meliponary_id
    WHERE f.voided_at IS NULL
      AND c.status IN ('active', 'weak', 'recovering')
      AND m.archived_at IS NULL
)
INSERT INTO scheduled_tasks (
    id, meliponary_id, colony_id, box_id, task_type, title,
    scheduled_for, status, priority, source_type, source_id
)
SELECT lower(hex(randomblob(16))), meliponary_id, colony_id, box_id,
       'feeding', 'Alimentar ' || colony_code,
       next_feeding_at, 'pending', 'normal', 'feeding', id
FROM ranked
WHERE rn = 1 AND next_feeding_at IS NOT NULL;

WITH ranked AS (
    SELECT r.id, r.box_id, r.colony_id, r.next_maintenance_at,
           b.meliponary_id, b.code AS box_code,
           ROW_NUMBER() OVER (
               PARTITION BY r.box_id
               ORDER BY r.maintained_at DESC, r.created_at DESC, r.id DESC
           ) AS rn
    FROM box_maintenance_records r
    JOIN boxes b ON b.id = r.box_id
    JOIN meliponaries m ON m.id = b.meliponary_id
    WHERE r.voided_at IS NULL
      AND b.status <> 'retired'
      AND m.archived_at IS NULL
)
INSERT INTO scheduled_tasks (
    id, meliponary_id, colony_id, box_id, task_type, title,
    scheduled_for, status, priority, source_type, source_id
)
SELECT lower(hex(randomblob(16))), meliponary_id, colony_id, box_id,
       'maintenance', 'Revisar caixa ' || box_code,
       next_maintenance_at, 'pending', 'normal', 'maintenance', id
FROM ranked
WHERE rn = 1 AND next_maintenance_at IS NOT NULL;
