import type { DashboardStats, View } from "../types";

type DashboardPageProps = { stats: DashboardStats; onNavigate: (view: View) => void; };

export function DashboardPage({ stats, onNavigate }: DashboardPageProps) {
  const cards = [
    ["Meliponários", stats.meliponaries], ["Espécies", stats.species], ["Colônias", stats.colonies], ["Caixas", stats.boxes], ["Inspeções", stats.inspections], ["Fotos", stats.photos], ["Eventos", stats.events], ["Divisões", stats.divisions], ["Alimentações", stats.feedings], ["Produção", stats.production], ["Movimentações", stats.movements], ["Documentos", stats.documents], ["Manutenções", stats.maintenance], ["Ciclo de vida", stats.lifecycle], ["Alertas", stats.alerts],
  ] as const;

  return (
    <div className="page-stack">
      <section className="page-heading dashboard-heading">
        <div><span className="eyebrow">Visão geral</span><h1>Seu plantel, com histórico de verdade.</h1><p>Cadastre a estrutura básica e avance para manejo, rastreabilidade e acompanhamento sem sair da aplicação.</p></div>
        <div className="quick-actions" aria-label="Ações rápidas">
          <button type="button" onClick={() => onNavigate("inspections")}>Nova inspeção</button>
          <button type="button" className="button-secondary" onClick={() => onNavigate("movements")}>Movimentar</button>
          <button type="button" className="button-secondary" onClick={() => onNavigate("alerts")}>Alertas</button>
        </div>
      </section>
      <section className="stats-grid" aria-label="Resumo do meliponário">{cards.map(([label, value]) => <article className="stat-card" key={label}><span>{label}</span><strong>{value}</strong></article>)}</section>
      <section className="panel getting-started">
        <div><span className="eyebrow">Fluxo operacional</span><h2>Da estrutura à rastreabilidade</h2><p>Os registros datados preservam o contexto histórico. Movimentações e documentos entram na mesma trilha do plantel sem duplicar a identidade da colônia.</p></div>
        <div className="step-list">
          <button type="button" onClick={() => onNavigate("colonies")}><span>1</span> Colônias</button>
          <button type="button" onClick={() => onNavigate("inspections")}><span>2</span> Inspeções</button>
          <button type="button" onClick={() => onNavigate("feeding")}><span>3</span> Alimentação</button>
          <button type="button" onClick={() => onNavigate("production")}><span>4</span> Produção</button>
          <button type="button" onClick={() => onNavigate("history")}><span>5</span> Histórico</button>
          <button type="button" onClick={() => onNavigate("genealogy")}><span>6</span> Divisões</button>
          <button type="button" onClick={() => onNavigate("movements")}><span>7</span> Movimentações</button>
          <button type="button" onClick={() => onNavigate("alerts")}><span>8</span> Alertas</button>
        </div>
      </section>
    </div>
  );
}
