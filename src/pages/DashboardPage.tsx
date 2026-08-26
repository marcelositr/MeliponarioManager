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
          <p>Cadastre a estrutura básica aqui e depois avance para manejo, inspeções e rastreabilidade.</p>
        </div>
        <div className="quick-actions" aria-label="Ações rápidas">
          <button type="button" onClick={() => onNavigate("colonies")}>Nova colônia</button>
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
          <h2>Monte a base em uma sequência simples</h2>
          <p>Cadastre o meliponário, as espécies que você trabalha, as caixas físicas e então as colônias. Depois, vincule cada colônia à sua caixa atual.</p>
        </div>
        <div className="step-list">
          <button type="button" onClick={() => onNavigate("meliponaries")}><span>1</span> Meliponários</button>
          <button type="button" onClick={() => onNavigate("species")}><span>2</span> Espécies</button>
          <button type="button" onClick={() => onNavigate("boxes")}><span>3</span> Caixas</button>
          <button type="button" onClick={() => onNavigate("colonies")}><span>4</span> Colônias</button>
        </div>
      </section>
    </div>
  );
}
