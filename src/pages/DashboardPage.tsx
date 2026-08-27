import type { DashboardStats, View } from "../types";

type DashboardPageProps = {
  stats: DashboardStats;
  onNavigate: (view: View) => void;
};

export function DashboardPage({ stats, onNavigate }: DashboardPageProps) {
  const cards = [
    ["Meliponários", stats.meliponaries],
    ["Espécies", stats.species],
    ["Colônias", stats.colonies],
    ["Caixas", stats.boxes],
    ["Inspeções", stats.inspections],
    ["Fotos", stats.photos],
    ["Eventos", stats.events],
    ["Divisões", stats.divisions],
    ["Alimentações", stats.feedings],
    ["Produção", stats.production],
    ["Movimentações", stats.movements],
    ["Documentos", stats.documents],
    ["Manutenções", stats.maintenance],
    ["Ciclo de vida", stats.lifecycle],
    ["Alertas", stats.alerts],
  ] as const;

  return (
    <div className="page-stack">
      <section className="page-heading dashboard-heading">
        <div>
          <span className="eyebrow">Visão geral</span>
          <h1>Seu plantel, com histórico de verdade.</h1>
          <p>Cadastre a estrutura básica e avance para o manejo sem sair da aplicação.</p>
        </div>
        <div className="quick-actions" aria-label="Ações rápidas">
          <button type="button" onClick={() => onNavigate("inspections")}>Nova inspeção</button>
          <button type="button" className="button-secondary" onClick={() => onNavigate("colonies")}>Nova colônia</button>
          <button type="button" className="button-secondary" onClick={() => onNavigate("boxes")}>Nova caixa</button>
        </div>
      </section>

      <section className="stats-grid" aria-label="Resumo do meliponário">
        {cards.map(([label, value]) => (
          <article className="stat-card" key={label}>
            <span>{label}</span>
            <strong>{value}</strong>
          </article>
        ))}
      </section>

      <section className="panel getting-started">
        <div>
          <span className="eyebrow">Primeiros passos</span>
          <h2>Da estrutura ao primeiro manejo</h2>
          <p>Cadastre o meliponário, as espécies, as caixas e as colônias. Depois de vincular cada colônia à caixa atual, registre as inspeções diretamente pela interface.</p>
        </div>
        <div className="step-list">
          <button type="button" onClick={() => onNavigate("meliponaries")}><span>1</span> Meliponários</button>
          <button type="button" onClick={() => onNavigate("species")}><span>2</span> Espécies</button>
          <button type="button" onClick={() => onNavigate("boxes")}><span>3</span> Caixas</button>
          <button type="button" onClick={() => onNavigate("colonies")}><span>4</span> Colônias</button>
          <button type="button" onClick={() => onNavigate("inspections")}><span>5</span> Inspeções</button>
        </div>
      </section>
    </div>
  );
}
