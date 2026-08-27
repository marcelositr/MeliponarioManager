import { useEffect, useState } from "react";
import { PageToolbar } from "../components/PageToolbar";
import { loadDashboardOverview } from "../lib/dashboard-api";
import type { DashboardOverview } from "../dashboard-types";
import type { DashboardStats, View } from "../types";

type DashboardPageProps = { stats: DashboardStats; onNavigate: (view: View) => void; };

export function DashboardPage({ stats, onNavigate }: DashboardPageProps) {
  const [overview, setOverview] = useState<DashboardOverview | null>(null);
  const [error, setError] = useState("");
  useEffect(() => { let cancelled = false; loadDashboardOverview().then((data) => { if (!cancelled) setOverview(data); }).catch(() => { if (!cancelled) setError("Não foi possível carregar os indicadores detalhados."); }); return () => { cancelled = true; }; }, [stats]);

  return <div className="page-stack">
    <PageToolbar title="Visão geral" description="Situação operacional do plantel com dados derivados dos registros atuais."><button type="button" onClick={() => onNavigate("inspections")}>Nova inspeção</button><button type="button" className="button-secondary" onClick={() => onNavigate("alerts")}>Ver alertas</button><button type="button" className="button-secondary" onClick={() => onNavigate("movements")}>Movimentações</button></PageToolbar>
    <section className="stats-grid executive-stats" aria-label="Indicadores principais"><Stat label="Colônias" value={stats.colonies} /><Stat label="Caixas" value={stats.boxes} /><Stat label="Alertas" value={stats.alerts} attention={stats.alerts > 0} /><Stat label="Inspeções" value={stats.inspections} /></section>
    {error && <div className="inline-notice">{error}</div>}
    {!overview && !error ? <div className="empty-list">Calculando visão operacional...</div> : overview && <>
      <div className="content-grid">
        <section className="panel"><div className="panel-heading"><h2>Situação do plantel</h2><p>Distribuição administrativa atual das colônias.</p></div><MetricTable items={overview.colonyStatuses} label={statusLabel} /></section>
        <section className="panel"><div className="panel-heading"><h2>Caixas</h2><p>Ocupação derivada do histórico aberto de colônia ↔ caixa.</p></div><div className="stats-grid"><Stat label="Ocupadas" value={overview.occupiedBoxes} /><Stat label="Ativas e livres" value={overview.freeBoxes} /></div><div className="dashboard-secondary-metrics"><span>{stats.maintenance} manutenções registradas</span><span>{stats.lifecycle} eventos de ciclo de vida</span></div></section>
      </div>
      <div className="content-grid">
        <section className="panel"><div className="panel-heading"><h2>Força da última inspeção</h2><p>Somente a inspeção mais recente determina a projeção.</p></div><MetricTable items={overview.inspectionStrengths} label={strengthLabel} /></section>
        <section className="panel"><div className="panel-heading"><h2>Distribuição por espécie</h2><p>Quantidade de colônias por referência do catálogo.</p></div><MetricTable items={overview.speciesDistribution} /></section>
      </div>
      <section className="panel wide-list"><div className="panel-heading"><h2>Pendências atuais</h2><p>Alertas derivados, sem persistir estado paralelo.</p></div>{overview.alerts.length === 0 ? <div className="empty-list">Nenhuma pendência atual.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Colônia</th><th>Alerta</th><th>Quando</th><th>Severidade</th><th>Detalhes</th></tr></thead><tbody>{overview.alerts.slice(0, 8).map((alert) => <tr key={alert.alertKey}><td><strong>{alert.colonyCode}</strong></td><td>{alert.title}</td><td>{alert.dueAt ? formatDateTime(alert.dueAt) : alertTypeLabel(alert.alertType)}</td><td><span className={`badge severity-${alert.severity}`}>{severityLabel(alert.severity)}</span></td><td>{alert.details || "—"}</td></tr>)}</tbody></table></div>}<div className="form-actions"><button type="button" className="button-secondary" onClick={() => onNavigate("alerts")}>Abrir central de alertas</button></div></section>
      <div className="content-grid">
        <section className="panel wide-list"><div className="panel-heading"><h2>Produção recente</h2><p>Últimas colheitas quantificadas.</p></div>{overview.recentProduction.length === 0 ? <div className="empty-list">Nenhuma produção registrada.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Colônia</th><th>Produto</th><th>Quantidade</th><th>Data</th></tr></thead><tbody>{overview.recentProduction.map((item, index) => <tr key={`${item.colonyCode}-${item.harvestedAt}-${index}`}><td><strong>{item.colonyCode}</strong></td><td>{productLabel(item.productType)}</td><td>{item.quantity} {item.unit}</td><td>{formatDateTime(item.harvestedAt)}</td></tr>)}</tbody></table></div>}</section>
        <section className="panel wide-list"><div className="panel-heading"><h2>Movimentações recentes</h2><p>Últimos deslocamentos registrados.</p></div>{overview.recentMovements.length === 0 ? <div className="empty-list">Nenhuma movimentação registrada.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Colônia</th><th>Tipo</th><th>Data</th><th>Destino</th></tr></thead><tbody>{overview.recentMovements.map((item, index) => <tr key={`${item.colonyCode}-${item.movedAt}-${index}`}><td><strong>{item.colonyCode}</strong></td><td>{movementLabel(item.movementType)}</td><td>{formatDateTime(item.movedAt)}</td><td>{item.destination || "—"}</td></tr>)}</tbody></table></div>}</section>
      </div>
    </>}
  </div>;
}

function Stat({ label, value, attention = false }: { label: string; value: number; attention?: boolean }) { return <article className={attention ? "stat-card attention" : "stat-card"}><span>{label}</span><strong>{value}</strong></article>; }
function MetricTable({ items, label = (value: string) => value }: { items: Array<{ label: string; count: number }>; label?: (value: string) => string }) { return items.length === 0 ? <div className="empty-list">Sem dados para este indicador.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Categoria</th><th>Quantidade</th></tr></thead><tbody>{items.map((item) => <tr key={item.label}><td>{label(item.label)}</td><td><strong>{item.count}</strong></td></tr>)}</tbody></table></div>; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function statusLabel(value: string) { const labels: Record<string, string> = { active: "Ativas", weak: "Fracas", recovering: "Em recuperação", inactive: "Inativas", lost: "Perdidas", transferred: "Transferidas" }; return labels[value] || value; }
function strengthLabel(value: string) { const labels: Record<string, string> = { strong: "Fortes", medium: "Médias", weak: "Fracas", unknown: "Sem avaliação" }; return labels[value] || value; }
function productLabel(value: string) { const labels: Record<string, string> = { honey: "Mel", pollen: "Pólen", propolis: "Própolis", wax: "Cera", cerumen: "Cerume", other: "Outro" }; return labels[value] || value; }
function movementLabel(value: string) { return value === "internal_transfer" ? "Transferência interna" : value === "external_transfer" ? "Transferência externa" : "Transporte"; }
function severityLabel(value: string) { return value === "critical" ? "Crítico" : value === "attention" ? "Atenção" : "Informativo"; }
function alertTypeLabel(value: string) { const labels: Record<string, string> = { inspection_due: "Inspeção pendente", feeding_due: "Alimentação pendente", weak_colony: "Colônia fraca" }; return labels[value] || value; }
