import { createPortal } from "react-dom";
import { useEffect, useId, useLayoutEffect, useRef, useState, type CSSProperties, type KeyboardEvent, type ReactNode } from "react";

type SecondaryAction = { label: string; onClick: () => void; disabled?: boolean; danger?: boolean };
type RecordActionsProps = { onOpen?: () => void; onEdit?: () => void; secondary?: SecondaryAction[]; busy?: boolean; children?: ReactNode };

export function RecordActions({ onOpen, onEdit, secondary = [], busy = false, children }: RecordActionsProps) {
  const available = secondary.filter((action) => !action.disabled);
  const [open, setOpen] = useState(false);
  const [menuStyle, setMenuStyle] = useState<CSSProperties>({ top: 0, left: 0, visibility: "hidden" });
  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const menuId = useId();

  function updatePosition() {
    const trigger = triggerRef.current;
    const menu = menuRef.current;
    if (!trigger || !menu) return;

    const triggerRect = trigger.getBoundingClientRect();
    const menuRect = menu.getBoundingClientRect();
    const viewportPadding = 8;
    const gap = 4;
    const maxLeft = Math.max(viewportPadding, window.innerWidth - menuRect.width - viewportPadding);
    const preferredLeft = triggerRect.right - menuRect.width;
    const left = Math.min(Math.max(preferredLeft, viewportPadding), maxLeft);

    let top = triggerRect.bottom + gap;
    const fitsBelow = top + menuRect.height <= window.innerHeight - viewportPadding;
    const fitsAbove = triggerRect.top - menuRect.height - gap >= viewportPadding;
    if (!fitsBelow && fitsAbove) top = triggerRect.top - menuRect.height - gap;
    else top = Math.min(top, Math.max(viewportPadding, window.innerHeight - menuRect.height - viewportPadding));

    setMenuStyle({ top: Math.round(top), left: Math.round(left), visibility: "visible" });
  }

  function focusItem(index: number) {
    const items = Array.from(menuRef.current?.querySelectorAll<HTMLButtonElement>("[role='menuitem']:not([disabled])") ?? []);
    items[index]?.focus();
  }

  useLayoutEffect(() => {
    if (!open) return;
    updatePosition();
    const frame = window.requestAnimationFrame(() => focusItem(0));
    return () => window.cancelAnimationFrame(frame);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const close = (event: MouseEvent) => {
      const target = event.target as Node;
      if (!rootRef.current?.contains(target) && !menuRef.current?.contains(target)) setOpen(false);
    };
    const reposition = () => updatePosition();
    document.addEventListener("mousedown", close);
    window.addEventListener("resize", reposition);
    window.addEventListener("scroll", reposition, true);
    return () => {
      document.removeEventListener("mousedown", close);
      window.removeEventListener("resize", reposition);
      window.removeEventListener("scroll", reposition, true);
    };
  }, [open]);

  function openMenu() {
    setMenuStyle({ top: 0, left: 0, visibility: "hidden" });
    setOpen(true);
  }

  function onTriggerKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    if (event.key === "ArrowDown" || event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      openMenu();
    }
  }

  function onItemKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    const items = Array.from(menuRef.current?.querySelectorAll<HTMLButtonElement>("[role='menuitem']:not([disabled])") ?? []);
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

  function runAction(action: SecondaryAction) {
    setOpen(false);
    triggerRef.current?.focus();
    action.onClick();
  }

  const menu = open && typeof document !== "undefined"
    ? createPortal(<div id={menuId} ref={menuRef} className="action-menu-popover" role="menu" aria-label="Mais ações" style={menuStyle}>
      {available.map((action) => <button key={action.label} type="button" role="menuitem" className={action.danger ? "danger-action" : undefined} onKeyDown={onItemKeyDown} onClick={() => runAction(action)} disabled={busy}>{action.label}</button>)}
    </div>, document.body)
    : null;

  return <div className="record-actions">
    {onOpen && <button className="table-action" type="button" onClick={onOpen} disabled={busy}>Abrir</button>}
    {onEdit && <button className="table-action" type="button" onClick={onEdit} disabled={busy}>Editar</button>}
    {children}
    {available.length > 0 && <div className="action-menu" ref={rootRef}>
      <button ref={triggerRef} className="action-menu-trigger" type="button" aria-label="Mais ações" title="Mais ações" aria-haspopup="menu" aria-controls={open ? menuId : undefined} aria-expanded={open} onClick={() => open ? setOpen(false) : openMenu()} onKeyDown={onTriggerKeyDown}>⋮</button>
      {menu}
    </div>}
  </div>;
}
