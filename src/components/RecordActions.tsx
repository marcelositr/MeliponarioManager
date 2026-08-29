import { useEffect, useRef, useState, type KeyboardEvent, type ReactNode } from "react";

type SecondaryAction = { label: string; onClick: () => void; disabled?: boolean; danger?: boolean };
type RecordActionsProps = { onOpen?: () => void; onEdit?: () => void; secondary?: SecondaryAction[]; busy?: boolean; children?: ReactNode };

export function RecordActions({ onOpen, onEdit, secondary = [], busy = false, children }: RecordActionsProps) {
  const available = secondary.filter((action) => !action.disabled);
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!open) return;
    const close = (event: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [open]);

  function focusItem(index: number) {
    const items = Array.from(rootRef.current?.querySelectorAll<HTMLButtonElement>("[role='menuitem']:not([disabled])") ?? []);
    items[index]?.focus();
  }

  function openMenu() {
    setOpen(true);
    window.requestAnimationFrame(() => focusItem(0));
  }

  function onTriggerKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    if (event.key === "ArrowDown" || event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      openMenu();
    }
  }

  function onItemKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    const items = Array.from(rootRef.current?.querySelectorAll<HTMLButtonElement>("[role='menuitem']:not([disabled])") ?? []);
    const index = items.indexOf(event.currentTarget);
    if (event.key === "Escape") {
      event.preventDefault();
      setOpen(false);
      triggerRef.current?.focus();
    } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const next = (index + (event.key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
      items[next]?.focus();
    } else if (event.key === "Home" || event.key === "End") {
      event.preventDefault();
      items[event.key === "Home" ? 0 : items.length - 1]?.focus();
    }
  }

  return <div className="record-actions">
    {onOpen && <button className="table-action" type="button" onClick={onOpen} disabled={busy}>Abrir</button>}
    {onEdit && <button className="table-action" type="button" onClick={onEdit} disabled={busy}>Editar</button>}
    {children}
    {available.length > 0 && <div className="action-menu" ref={rootRef}>
      <button ref={triggerRef} className="action-menu-trigger" type="button" aria-label="Mais ações" title="Mais ações" aria-haspopup="menu" aria-expanded={open} onClick={() => open ? setOpen(false) : openMenu()} onKeyDown={onTriggerKeyDown}>⋮</button>
      {open && <div className="action-menu-popover" role="menu" aria-label="Mais ações">
        {available.map((action) => <button key={action.label} type="button" role="menuitem" className={action.danger ? "danger-action" : undefined} onKeyDown={onItemKeyDown} onClick={() => { setOpen(false); window.requestAnimationFrame(() => triggerRef.current?.focus()); action.onClick(); }} disabled={busy}>{action.label}</button>)}
      </div>}
    </div>}
  </div>;
}
