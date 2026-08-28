import { useEffect, useMemo, useState } from "react";
import { PageToolbar } from "../components/PageToolbar";
import type { DashboardOverview } from "../dashboard-types";
import { listAlerts } from "../lib/api";
import { getAgendaSummary } from "../lib/agenda-api";
import type { AgendaSummary } from "../lib/agenda-types";
import { loadDashboardOverview } from "../lib/dashboard-api";
import type { Alert, CoreData, DashboardStats, View } from "../types";

type DashboardPageProps = {
  stats: DashboardStats;
  data: CoreData;
  activeMeliponaryId: string;
  onNavigate: (view: View) => void;
};

const emptyAgenda: AgendaSummary = { overdue: 0, today: 0, nextSevenDays: 0, future: 0 };

export function DashboardPage({ stats, data, activeMeliponaryId, onNavigate }: DashboardPageProps) {
  const [overview, setOverview] = useState<DashboardOverview | null>(null);
  const [agenda, setAgenda] = useState<AgendaSummary>(emptyAgenda);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;
    setError("");
    Promise.all([
      loadDashboardOverview(),
      getAgendaSummary(activeMeliponaryId || undefined),
      listAlerts(),
    ]).then(([nextOverview, nextAgenda, nextAlerts]) => {
      if (cancelled) return;
      setOverview(nextOverview);
      setAgenda(nextAgenda);
      setAlerts(nextAlerts);
    }).catch(() => {
      if (!cancelled) setError("Não foi possível carregar todos os indicadores operacionais.");
    });
    return () => { cancelled = true; };
  }, [activeMeliponaryId, stats]);

  const scopedColonies = useMemo(
    () => activeMeliponaryId ? data.colonies.filter((item) => item.meliponaryId === activeMeliponaryId) : data.colonies,
    [activeMeliponaryId, data.colonies],
  );
  const scopedBoxes = useMemo(
    () => activeMeliponaryId ? data.boxes.filter((item) => item.meliponaryId === activeMeliponaryId) : data.boxes,
    [activeMeliponaryId, data.boxes],
  );
  const scopedAlerts = useMemo(
    () => activeMeliponaryId ? alerts.filter((item) => item.meliponaryId === activeMeliponaryId) : alerts,
    [activeMeliponaryId, alerts],
  );
  const statusCounts = useMemo(() => countBy(scopedColonies.map((item) => normalizeStatus(item.status))), [scopedColonies]);
  const speciesNames = useMemo(() => new Map(data.species.map((item) => [item.id, item.commonName])), [data.species]);
  const speciesCounts = useMemo(() => countBy(scopedColonies.map((item) => speciesNames.get(item.speciesId) || "Espécie não encontrada")), [scopedColonies, speciesNames]);
  const occupiedBoxes = scopedBoxes.filter((item) => Boolean(item.currentColonyCode)).length;
  const freeBoxes = scopedBoxes.filter((item) => item.status === "active" && !item.currentColonyCode).length;

  return <div className="page-stack">
    <PageToolbar title="Visão geral" description={activeMeliponaryId ? "Situação operacional do meliponário selecionado." : "Situação operacional consolidada de todos os meliponários."}>
      <button type="button" onClick={() => onNavigate("agenda")}>Abrir Agenda</button>
      <button type="button" className="button-secondary" onClick={() => onNavigate("alerts")}>Ver alertas</button>
      <button type="button" className="button-secondary" onClick={() => onNavigate("inspections")}>Nova inspeção</button>
    </PageToolbar>

    <section className="stats-grid executive-stats" aria-label="Indicadores principais">
      <Stat label="Colônias" value={scopedColonies.length} />
      <Stat label="Caixas" value={scopedBoxes.length} />
      <Stat label="Alertas" value={scopedAlerts.length} attention={scopedAlerts.length > 0} />
      <Stat label="Agenda atrasada" value={agenda.overdue} attention={agenda.overdue > 0} />
    </section>

    {error && <div className="inline-notice">{error}</div>}

    <section className="panel">
      <div className="panel-heading"><h2>Agenda operacional</h2><p>Compromissos futuros e atrasados são separados dos fatos já realizados.</p></div>
      <div className="stats-grid">
        <Stat label="Atrasadas" value={agenda.overdue} attention={agenda.overdue > 0} />
        <Stat label="Hoje" value={agenda.today} />
        <Stat label="Próximos 7 dias" value={agenda.nextSevenDays} />
        <Stat label="Futuras" value={agenda.future} />
      </div>
      <div className="form-actions"><button className="button-secondary" type="button" onClick={() => onNavigate("agenda")}>Gerenciar compromissos</button></div>
    </section>

    <div className="content-grid">
      <section className="panel"><div className="panel-heading"><h2>Situação do plantel</h2><p>Distribuição administrativa no contexto selecionado.</p></div><MetricTable items={statusCounts} label={statusLabel} /></section>
      <section className="panel"><div className="panel-heading"><h2>Caixas</h2><p>Ocupação atual derivada dos registros de colônia ↔ caixa.</p></div><div className="stats-grid"><Stat label="Ocupadas" value={occupiedBoxes} /><Stat label="Ativas e livres" value={freeBoxes} /></div></section>
    </div>

    <div className="content-grid">
      <section className="panel"><div className="panel-heading"><h2>Distribuição por espécie</h2><p>Quantidade de colônias no contexto operacional atual.</p></div><MetricTable items={speciesCounts} /></section>
      {!activeMeliponaryId && overview ? <section className="panel"><div className="panel-heading"><h2>Força da última inspeção</h2><p>Projeção consolidada derivada das inspeções mais recentes.</p></div><MetricTable items={overview.inspectionStrengths} label={strengthLabel} /></section> : <section className="panel"><div className="panel-heading"><h2>Contexto aplicado</h2><p>Indicadores sem contrato de escopo por meliponário não são misturados com a visão filtrada.</p></div><div className="empty-list">Produção e movimentações recentes continuam disponíveis em suas telas próprias, já filtradas pelo contexto ativo.</div></section>}
    </div>

    <section className="panel wide-list">
      <div className="panel-heading"><h2>Pendências atuais</h2><p>Alertas derivados levam diretamente à Agenda ou ao manejo recomendado.</p></div>
      {scopedAlerts.length === 0 ? <div className="empty-list">Nenhuma pendência atual.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Contexto</th><th>Alerta</th><th>Quando</th><th>Severidade</th><th>Ação</th></tr></thead><tbody>{scopedAlerts.slice(0, 8).map((alert) => <tr key={alert.alertKey}><td><strong>{alert.colonyCode || alert.boxCode || "Meliponário"}</strong></td><td>{alert.title}</td><td>{alert.dueAt ? formatDateTime(alert.dueAt) : alertTypeLabel(alert.alertType)}</td><td><span className={`badge severity-${alert.severity}`}>{severityLabel(alert.severity)}</span></td><td><button className="button-secondary" type="button" onClick={() => onNavigate(alert.taskId ? "agenda" : recommendedView(alert.recommendedAction))}>{alert.taskId ? "Abrir Agenda" : recommendedLabel(alert.recommendedAction)}</button></td></tr>)}</tbody></table></div>}
      <div className="form-actions"><button type="button" className="button-secondary" onClick={() => onNavigate("alerts")}>Abrir central de alertas</button></div>
    </section>

    {!activeMeliponaryId && overview && <div className="content-grid">
      <section className="panel wide-list"><div className="panel-heading"><h2>Produção recente</h2><p>Últimas colheitas quantificadas no consolidado.</p></div>{overview.recentProduction.length === 0 ? <div className="empty-list">Nenhuma produção registrada.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Colônia</th><th>Produto</th><th>Quantidade</th><th>Data</th></tr></thead><tbody>{overview.recentProduction.map((item, index) => <tr key={`${item.colonyCode}-${item.harvestedAt}-${index}`}><td><strong>{item.colonyCode}</strong></td><td>{productLabel(item.productType)}</td><td>{item.quantity} {item.unit}</td><td>{formatDateTime(item.harvestedAt)}</td></tr>)}</tbody></table></div>}</section>
      <section className="panel wide-list"><div className="panel-heading"><h2>Movimentações recentes</h2><p>Últimos deslocamentos registrados no consolidado.</p></div>{overview.recentMovements.length === 0 ? <div className="empty-list">Nenhuma movimentação registrada.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Colônia</th><th>Tipo</th><th>Data</th><th>Destino</th></tr></thead><tbody>{overview.recentMovements.map((item, index) => <tr key={`${item.colonyCode}-${item.movedAt}-${index}`}><td><strong>{item.colonyCode}</strong></td><td>{movementLabel(item.movementType)}</td><td>{formatDateTime(item.movedAt)}</td><td>{item.destination || "—"}</td></tr>)}</tbody></table></div>}</section>
    </div>}
  </div>;
}

function Stat({ label, value, attention = false }: { label: string; value: number; attention?: boolean }) { return <article className={attention ? "stat-card attention" : "stat-card"}><span>{label}</span><strong>{value}</strong></article>; }
function MetricTable({ items, label = (value: string) => value }: { items: Array<{ label: string; count: number }>; label?: (value: string) => string }) { return items.length === 0 ? <div className="empty-list">Sem dados para este indicador.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Categoria</th><th>Quantidade</th></tr></thead><tbody>{items.map((item) => <tr key={item.label}><td>{label(item.label)}</td><td><strong>{item.count}</strong></td></tr>)}</tbody></table></div>; }
function countBy(values: string[]) { const counts = new Map<string, number>(); for (const value of values) counts.set(value, (counts.get(value) || 0) + 1); return [...counts.entries()].map(([label, count]) => ({ label, count })); }
function normalizeStatus(value: string) { return value === "weak" || value === "recovering" ? "active" : value; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function statusLabel(value: string) { const labels: Record<string, string> = { active: "Ativas", inactive: "Inativas", lost: "Perdidas", transferred: "Transferidas" }; return labels[value] || value; }
function strengthLabel(value: string) { const labels: Record<string, string> = { strong: "Fortes", medium: "Médias", weak: "Fracas", unknown: "Sem avaliação" }; return labels[value] || value; }
function productLabel(value: string) { const labels: Record<string, string> = { honey: "Mel", pollen: "Pólen", propolis: "Própolis", wax: "Cera", cerumen: "Cerume", other: "Outro" }; return labels[value] || value; }
function movementLabel(value: string) { return value === "internal_transfer" ? "Transferência interna" : value === "external_transfer" ? "Transferência externa" : "Transporte"; }
function severityLabel(value: string) { return value === "critical" ? "Crítico" : value === "attention" ? "Atenção" : "Informativo"; }
function alertTypeLabel(value: string) { const labels: Record<string, string> = { inspection_due: "Inspeção pendente", feeding_due: "Alimentação pendente", maintenance_due: "Manutenção pendente", weak_colony: "Colônia fraca" }; return labels[value] || value; }
function recommendedView(value: string): View { if (value === "register_feeding") return "feeding"; if (value === "register_maintenance") return "assets"; return "inspections"; }
function recommendedLabel(value: string) { if (value === "register_feeding") return "Alimentar"; if (value === "register_maintenance") return "Manutenção"; return "Inspecionar"; }
