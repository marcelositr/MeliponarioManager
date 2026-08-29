import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState, type KeyboardEvent as ReactKeyboardEvent, type ReactNode } from "react";
import type { ThemeMode } from "../lib/ui-preferences";
import type { View } from "../types";
import { Icon } from "./Icon";

type TopMenuProps = {
  theme: ThemeMode;
  onThemeChange: (theme: ThemeMode) => void;
  sidebarCollapsed: boolean;
  onToggleSidebar: () => void;
  onNavigate: (view: View) => void;
  onRefresh: () => void;
  refreshDisabled?: boolean;
  onOpenAbout: () => void;
};
type MenuName = "file" | "view" | "tools" | "help";
const menuOrder: MenuName[] = ["file", "view", "tools", "help"];
const menuItemSelector = "[role='menuitem']:not([disabled]), [role='menuitemradio']:not([disabled])";

export function TopMenu({ theme, onThemeChange, sidebarCollapsed, onToggleSidebar, onNavigate, onRefresh, refreshDisabled = false, onOpenAbout }: TopMenuProps) {
  const [open, setOpen] = useState<MenuName | null>(null);
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const close = (event: MouseEvent) => { if (rootRef.current && !rootRef.current.contains(event.target as Node)) setOpen(null); };
    const escape = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || !open) return;
      const active = rootRef.current?.querySelector<HTMLElement>(`[data-menu-trigger="${open}"]`);
      setOpen(null);
      active?.focus();
    };
    document.addEventListener("mousedown", close);
    window.addEventListener("keydown", escape);
    return () => { document.removeEventListener("mousedown", close); window.removeEventListener("keydown", escape); };
  }, [open]);

  function toggle(name: MenuName) { setOpen((current) => current === name ? null : name); }
  function action(fn: () => void) { return () => { setOpen(null); fn(); }; }
  function moveTrigger(name: MenuName, direction: -1 | 1) {
    const index = menuOrder.indexOf(name);
    const next = menuOrder[(index + direction + menuOrder.length) % menuOrder.length];
    rootRef.current?.querySelector<HTMLElement>(`[data-menu-trigger="${next}"]`)?.focus();
    if (open) setOpen(next);
  }

  return <div className="menu-bar" ref={rootRef} role="menubar" aria-label="Menu principal">
    <div className="menu-brand" aria-hidden="true">MeliponarioManager</div>
    <Menu name="file" label="Arquivo" open={open === "file"} onToggle={() => toggle("file")} onMove={moveTrigger}><MenuItem label="Backup" onClick={action(() => onNavigate("data"))} /><MenuItem label="Restaurar…" onClick={action(() => onNavigate("data"))} /><MenuItem label="Exportar" onClick={action(() => onNavigate("data"))} /><MenuItem label="Relatórios" onClick={action(() => onNavigate("data"))} /><div className="menu-separator" role="separator" /><MenuItem label="Sair" onClick={action(() => { void getCurrentWindow().close(); })} /></Menu>
    <Menu name="view" label="Exibir" open={open === "view"} onToggle={() => toggle("view")} onMove={moveTrigger}><MenuItem label="Tema claro" radio checked={theme === "light"} icon="sun" onClick={action(() => onThemeChange("light"))} /><MenuItem label="Tema escuro" radio checked={theme === "dark"} icon="moon" onClick={action(() => onThemeChange("dark"))} /><MenuItem label="Seguir sistema" radio checked={theme === "system"} icon="system" onClick={action(() => onThemeChange("system"))} /><div className="menu-separator" role="separator" /><MenuItem label={sidebarCollapsed ? "Expandir sidebar" : "Recolher sidebar"} onClick={action(onToggleSidebar)} /><MenuItem label="Atualizar" icon="refresh" disabled={refreshDisabled} onClick={action(onRefresh)} /></Menu>
    <Menu name="tools" label="Ferramentas" open={open === "tools"} onToggle={() => toggle("tools")} onMove={moveTrigger}><MenuItem label="Dados" icon="database" onClick={action(() => onNavigate("data"))} /></Menu>
    <Menu name="help" label="Ajuda" open={open === "help"} onToggle={() => toggle("help")} onMove={moveTrigger}><MenuItem label="Sobre…" icon="info" onClick={action(onOpenAbout)} /></Menu>
  </div>;
}

function Menu({ name, label, open, onToggle, onMove, children }: { name: MenuName; label: string; open: boolean; onToggle: () => void; onMove: (name: MenuName, direction: -1 | 1) => void; children: ReactNode }) {
  function onTriggerKeyDown(event: ReactKeyboardEvent<HTMLButtonElement>) {
    if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
      event.preventDefault();
      onMove(name, event.key === "ArrowRight" ? 1 : -1);
      return;
    }
    if (event.key === "ArrowDown" || event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      const root = event.currentTarget.parentElement;
      if (!open) onToggle();
      window.requestAnimationFrame(() => root?.querySelector<HTMLElement>(`[role='menu'] ${menuItemSelector}`)?.focus());
    }
  }
  return <div className="menu-root"><button data-menu-trigger={name} className={open ? "menu-trigger open" : "menu-trigger"} role="menuitem" type="button" onClick={onToggle} onKeyDown={onTriggerKeyDown} aria-haspopup="menu" aria-expanded={open}>{label}</button>{open && <div className="menu-popover" role="menu" aria-label={label}>{children}</div>}</div>;
}

function MenuItem({ label, onClick, disabled = false, checked = false, radio = false, icon }: { label: string; onClick?: () => void; disabled?: boolean; checked?: boolean; radio?: boolean; icon?: "sun" | "moon" | "system" | "refresh" | "database" | "info" }) {
  function onKeyDown(event: ReactKeyboardEvent<HTMLButtonElement>) {
    if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
      event.preventDefault();
      const bar = event.currentTarget.closest<HTMLElement>("[role='menubar']");
      const roots = Array.from(bar?.querySelectorAll<HTMLElement>(".menu-root") ?? []);
      const currentRoot = event.currentTarget.closest<HTMLElement>(".menu-root");
      const index = currentRoot ? roots.indexOf(currentRoot) : -1;
      if (index < 0 || roots.length === 0) return;
      const next = roots[(index + (event.key === "ArrowRight" ? 1 : -1) + roots.length) % roots.length];
      const trigger = next.querySelector<HTMLButtonElement>("[data-menu-trigger]");
      trigger?.click();
      window.requestAnimationFrame(() => next.querySelector<HTMLElement>(`[role='menu'] ${menuItemSelector}`)?.focus());
      return;
    }
    if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const menu = event.currentTarget.closest("[role='menu']");
    const items = Array.from(menu?.querySelectorAll<HTMLButtonElement>(menuItemSelector) ?? []);
    const index = items.indexOf(event.currentTarget);
    const next = event.key === "Home" ? 0 : event.key === "End" ? items.length - 1 : (index + (event.key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
    items[next]?.focus();
  }
  return <button className="menu-item" role={radio ? "menuitemradio" : "menuitem"} type="button" onClick={onClick} onKeyDown={onKeyDown} disabled={disabled} aria-checked={radio ? checked : undefined}><span className="menu-item-mark">{checked ? <Icon name="check" /> : icon ? <Icon name={icon} /> : null}</span><span>{label}</span></button>;
}
