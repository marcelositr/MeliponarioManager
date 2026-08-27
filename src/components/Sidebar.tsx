import type { View } from "../types";

type SidebarProps = {
  activeView: View;
  onNavigate: (view: View) => void;
  connectionStatus: string;
};

const items: Array<{ view: View; label: string; short: string }> = [
  { view: "dashboard", label: "Visão geral", short: "VG" },
  { view: "meliponaries", label: "Meliponários", short: "ME" },
  { view: "species", label: "Espécies", short: "ES" },
  { view: "colonies", label: "Colônias", short: "CO" },
  { view: "boxes", label: "Caixas", short: "CX" },
  { view: "inspections", label: "Inspeções", short: "IN" },
  { view: "feeding", label: "Alimentação", short: "AL" },
  { view: "production", label: "Produção", short: "PR" },
];

export function Sidebar({ activeView, onNavigate, connectionStatus }: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="brand-block">
        <div className="brand-mark">M</div>
        <div>
          <strong>MeliponarioManager</strong>
          <span>Gestão local do plantel</span>
        </div>
      </div>

      <nav className="side-nav" aria-label="Navegação principal">
        {items.map((item) => (
          <button
            className={activeView === item.view ? "nav-item active" : "nav-item"}
            key={item.view}
            onClick={() => onNavigate(item.view)}
            aria-current={activeView === item.view ? "page" : undefined}
            type="button"
          >
            <span className="nav-short">{item.short}</span>
            <span>{item.label}</span>
          </button>
        ))}
      </nav>

      <div className="sidebar-footer">
        <span className="status-dot" aria-hidden="true" />
        <div>
          <strong>Banco local</strong>
          <span>{connectionStatus}</span>
        </div>
      </div>
    </aside>
  );
}
