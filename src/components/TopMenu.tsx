import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState, type ReactNode } from "react";
import type { View } from "../types";
import { Icon } from "./Icon";

export type ThemeMode = "light" | "dark" | "system";

type TopMenuProps = {
  theme: ThemeMode;
  onThemeChange: (theme: ThemeMode) => void;
  sidebarCollapsed: boolean;
  onToggleSidebar: () => void;
  onNavigate: (view: View) => void;
  onRefresh: () => void;
  onOpenAbout: () => void;
};
type MenuName = "file" | "edit" | "view" | "tools" | "help";

export function TopMenu({ theme, onThemeChange, sidebarCollapsed, onToggleSidebar, onNavigate, onRefresh, onOpenAbout }: TopMenuProps) {
  const [open, setOpen] = useState<MenuName | null>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const close = (event: MouseEvent) => { if (rootRef.current && !rootRef.current.contains(event.target as Node)) setOpen(null); };
    const escape = (event: KeyboardEvent) => { if (event.key === "Escape") setOpen(null); };
    document.addEventListener("mousedown", close); window.addEventListener("keydown", escape);
    return () => { document.removeEventListener("mousedown", close); window.removeEventListener("keydown", escape); };
  }, []);
  function toggle(name: MenuName) { setOpen((current) => current === name ? null : name); }
  function action(fn: () => void) { return () => { setOpen(null); fn(); }; }

  return <div className="menu-bar" ref={rootRef}>
    <div className="menu-brand">MeliponarioManager</div>
    <Menu label="Arquivo" open={open === "file"} onToggle={() => toggle("file")}><MenuItem label="Backup" onClick={action(() => onNavigate("data"))} /><MenuItem label="Restaurar" onClick={action(() => onNavigate("data"))} /><MenuItem label="Exportar" onClick={action(() => onNavigate("data"))} /><MenuItem label="Relatórios" onClick={action(() => onNavigate("data"))} /><div className="menu-separator" /><MenuItem label="Sair" onClick={action(() => { void getCurrentWindow().close(); })} /></Menu>
    <Menu label="Editar" open={open === "edit"} onToggle={() => toggle("edit")}><MenuItem label="Novo registro" disabled /><MenuItem label="Editar registro" disabled /></Menu>
    <Menu label="Exibir" open={open === "view"} onToggle={() => toggle("view")}><MenuItem label="Tema claro" checked={theme === "light"} icon="sun" onClick={action(() => onThemeChange("light"))} /><MenuItem label="Tema escuro" checked={theme === "dark"} icon="moon" onClick={action(() => onThemeChange("dark"))} /><MenuItem label="Seguir sistema" checked={theme === "system"} icon="system" onClick={action(() => onThemeChange("system"))} /><div className="menu-separator" /><MenuItem label={sidebarCollapsed ? "Expandir sidebar" : "Recolher sidebar"} onClick={action(onToggleSidebar)} /><MenuItem label="Atualizar" icon="refresh" onClick={action(onRefresh)} /></Menu>
    <Menu label="Ferramentas" open={open === "tools"} onToggle={() => toggle("tools")}><MenuItem label="Dados" icon="database" onClick={action(() => onNavigate("data"))} /></Menu>
    <Menu label="Ajuda" open={open === "help"} onToggle={() => toggle("help")}><MenuItem label="Sobre" icon="info" onClick={action(onOpenAbout)} /></Menu>
  </div>;
}

function Menu({ label, open, onToggle, children }: { label: string; open: boolean; onToggle: () => void; children: ReactNode }) {
  return <div className="menu-root"><button className={open ? "menu-trigger open" : "menu-trigger"} type="button" onClick={onToggle} aria-expanded={open}>{label}</button>{open && <div className="menu-popover">{children}</div>}</div>;
}

function MenuItem({ label, onClick, disabled = false, checked = false, icon }: { label: string; onClick?: () => void; disabled?: boolean; checked?: boolean; icon?: "sun" | "moon" | "system" | "refresh" | "database" | "info" }) {
  return <button className="menu-item" type="button" onClick={onClick} disabled={disabled}><span className="menu-item-mark">{checked ? <Icon name="check" /> : icon ? <Icon name={icon} /> : null}</span><span>{label}</span></button>;
}
