import type { View } from "../types";
import { Icon, type IconName } from "./Icon";

type SidebarProps = {
  activeView: View;
  onNavigate: (view: View) => void;
  collapsed: boolean;
  onToggle: () => void;
};

type NavItem = { view: View; label: string; icon: IconName; title?: string };
type NavGroup = { label: string; items: NavItem[] };

const groups: NavGroup[] = [
  { label: "Operação", items: [
    { view: "dashboard", label: "Visão geral", icon: "dashboard" },
    { view: "alerts", label: "Alertas", icon: "alerts" },
  ]},
  { label: "Plantel", items: [
    { view: "meliponaries", label: "Meliponários", icon: "meliponary" },
    { view: "colonies", label: "Colônias", icon: "colony" },
    { view: "boxes", label: "Caixas", icon: "box" },
    { view: "species", label: "Espécies", icon: "species" },
  ]},
  { label: "Manejo", items: [
    { view: "inspections", label: "Inspeções", icon: "inspection" },
    { view: "feeding", label: "Alimentação", icon: "feeding" },
    { view: "production", label: "Produção", icon: "production" },
    { view: "assets", label: "Manutenção", icon: "maintenance", title: "Manutenção de caixas e acesso transitório às fotos" },
  ]},
  { label: "Rastreabilidade", items: [
    { view: "history", label: "Histórico", icon: "history" },
    { view: "genealogy", label: "Divisões e genealogia", icon: "genealogy" },
    { view: "movements", label: "Movimentações", icon: "movement" },
    { view: "lifecycle", label: "Ciclo de vida", icon: "lifecycle" },
  ]},
  { label: "Administração", items: [
    { view: "data", label: "Dados e relatórios", icon: "data" },
  ]},
];

export function Sidebar({ activeView, onNavigate, collapsed, onToggle }: SidebarProps) {
  return (
    <aside className={collapsed ? "sidebar collapsed" : "sidebar"} aria-label="Navegação principal">
      <div className="sidebar-brand">
        <div className="brand-mark" aria-hidden="true">M</div>
        <div className="brand-copy"><strong>MeliponarioManager</strong><span>Gestão do plantel</span></div>
        <button className="icon-button sidebar-toggle" type="button" onClick={onToggle} aria-label={collapsed ? "Expandir sidebar" : "Recolher sidebar"}><Icon name="menu" /></button>
      </div>
      <nav className="side-nav">
        {groups.map((group) => (
          <section className="nav-group" key={group.label}>
            <div className="nav-group-label">{group.label}</div>
            {group.items.map((item) => (
              <button
                key={item.view}
                type="button"
                className={activeView === item.view ? "nav-item active" : "nav-item"}
                onClick={() => onNavigate(item.view)}
                aria-current={activeView === item.view ? "page" : undefined}
                title={collapsed ? item.label : item.title}
              >
                <span className="nav-icon"><Icon name={item.icon} /></span>
                <span className="nav-label">{item.label}</span>
              </button>
            ))}
          </section>
        ))}
      </nav>
    </aside>
  );
}
