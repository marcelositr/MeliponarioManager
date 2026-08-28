import type { ReactNode } from "react";

type SecondaryAction = { label: string; onClick: () => void; disabled?: boolean; danger?: boolean };
type RecordActionsProps = { onOpen?: () => void; onEdit?: () => void; secondary?: SecondaryAction[]; busy?: boolean; children?: ReactNode };

export function RecordActions({ onOpen, onEdit, secondary = [], busy = false, children }: RecordActionsProps) {
  const available = secondary.filter((action) => !action.disabled);
  return <div className="record-actions">
    {onOpen && <button className="table-action" type="button" onClick={onOpen} disabled={busy}>Abrir</button>}
    {onEdit && <button className="table-action" type="button" onClick={onEdit} disabled={busy}>Editar</button>}
    {children}
    {available.length > 0 && <details className="action-menu">
      <summary aria-label="Mais ações" title="Mais ações">⋮</summary>
      <div className="action-menu-popover">
        {available.map((action) => <button key={action.label} type="button" className={action.danger ? "danger-action" : undefined} onClick={(event) => { event.currentTarget.closest("details")?.removeAttribute("open"); action.onClick(); }} disabled={busy}>{action.label}</button>)}
      </div>
    </details>}
  </div>;
}
