import { useEffect, useState } from "react";
import { loadDashboardOverview } from "../lib/dashboard-api";
import type { DashboardOverview } from "../dashboard-types";
import type { DashboardStats, View } from "../types";

type DashboardPageProps = { stats: DashboardStats; onNavigate: (view: View) => void; };

export function DashboardPage({ stats, onNavigate }: DashboardPageProps) {
  const [overview, setOverview] = useState<DashboardOverview | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;
    loadDashboardOverview()
      .then((data) => { if (!cancelled) setOverview(data); })
      .catch(() => { if (!cancelled) setError("Não foi possível carregar os indicadores detalhados."); });
    return () => { cancelled = true; };
  }, [stats]);

  const cards = [
    ["Meliponários", stats.meliponaries], ["Espécies", stats.species], ["Colônias", stats.colonies], ["Caixas", stats.boxes], ["Inspeções", stats.inspections], ["Fotos", stats.photos], ["Eventos", stats.events], ["Divisões", stats.divisions], ["Alimentações", stats.feedings], ["Produção", stats.production], ["Movimentações", stats.movements], ["Documentos", stats.documents], ["Manutenções", stats.maintenance], ["Ciclo de vida", stats.lifecycle], ["Alertas", stats.alerts],
  ] as const;

  return (
    <div className="page-stack">
      <section className="page-heading dashboard-heading">
        <div><span className="eyebrow">Visão geral</span><h1>Seu plantel, em estado atual.</h1><p>Os indicadores abaixo são derivados dos registros reais e da última informação disponível para cada colônia.</p></div>
        <div className="quick-actions" aria-label="Ações rápidas"><button type="button" onClick={() => onNavigate("inspections")}>Nova inspeção</button><button type="button" className="button-secondary" onClick={() => onNavigate("alerts")}>Alertas</button><button type="button" className="button-secondary" onClick={() => onNavigate("movements")}>Movimentar</button></div>
      </section>

      <section className="stats-grid" aria-label="Contadores gerais">{cards.map(([label, value]) => <article className="stat-card" key={label}><span>{label}</span><strong>{value}</strong></article>)}</section>

      {error && <div className="inline-notice">{error}</div>}
      {!overview && !error ? <div className="empty-list">Calculando visão operacional...</div> : overview && <>
        <div className="content-grid">
          <section className="panel"><div className="panel-heading"><h2>Situação do plantel</h2><p>Distribuição pelo status atual de cada colônia.</p></div><MetricList items={overview.colonyStatuses} label={statusLabel} /></section>
          <section className="panel"><div className="panel-heading"><h2>Força da última inspeção</h2><p>Considera somente colônias ativas, fracas ou em recuperação.</p></div><MetricList items={overview.inspectionStrengths} label={strengthLabel} /></section>
        </div>

        <div className="content-grid">
          <section className="panel"><div className="panel-heading"><h2>Distribuição por espécie</h2><p>Quantidade de colônias cadastradas para cada espécie.</p></div><MetricList items={overview.speciesDistribution} /></section>
          <section className="panel"><div className="panel-heading"><h2>Ocupação das caixas</h2><p>Ocupação é derivada do histórico aberto de colônia ↔ caixa.</p></div><div className="stats-grid"><article className="stat-card"><span>Ocupadas</span><strong>{overview.occupiedBoxes}</strong></article><article className="stat-card"><span>Ativas e livres</span><strong>{overview.freeBoxes}</strong></article></div></section>
        </div>

        <div className="content-grid">
          <section className="panel list-panel"><div className="panel-heading"><h2>Pendências atuais</h2><p>Alertas derivados das últimas inspeções, alimentações e situação da colônia.</p></div>{overview.alerts.length === 0 ? <div className="empty-list">Nenhuma pendência atual.</div> : <div className="record-list">{overview.alerts.slice(0, 5).map((alert) => <article className="record-card" key={alert.alertKey}><div className="record-title-row"><div><strong>{alert.colonyCode} · {alert.title}</strong><span>{alert.dueAt ? formatDateTime(alert.dueAt) : alertTypeLabel(alert.alertType)}</span></div><span className="badge">{severityLabel(alert.severity)}</span></div>{alert.details && <p>{alert.details}</p>}</article>)}</div>}<div className="form-actions"><button type="button" className="button-secondary" onClick={() => onNavigate("alerts")}>Ver todos os alertas</button></div></section>
          <section className="panel list-panel"><div className="panel-heading"><h2>Produção recente</h2><p>Últimos registros de colheita do plantel.</p></div>{overview.recentProduction.length === 0 ? <div className="empty-list">Nenhuma produção registrada.</div> : <div className="record-list">{overview.recentProduction.map((item, index) => <article className="record-card" key={`${item.colonyCode}-${item.harvestedAt}-${index}`}><div className="record-title-row"><div><strong>{item.colonyCode} · {productLabel(item.productType)}</strong><span>{formatDateTime(item.harvestedAt)}</span></div><span className="badge">{item.quantity} {item.unit}</span></div></article>)}</div>}</section>
        </div>

        <section className="panel list-panel"><div className="panel-heading"><h2>Movimentações recentes</h2><p>Últimos deslocamentos registrados no histórico do plantel.</p></div>{overview.recentMovements.length === 0 ? <div className="empty-list">Nenhuma movimentação registrada.</div> : <div className="record-list">{overview.recentMovements.map((item, index) => <article className="record-card" key={`${item.colonyCode}-${item.movedAt}-${index}`}><div className="record-title-row"><div><strong>{item.colonyCode} · {movementLabel(item.movementType)}</strong><span>{formatDateTime(item.movedAt)}{item.destination ? ` · ${item.destination}` : ""}</span></div></div></article>)}</div>}</section>
      </>}
    </div>
  );
}

function MetricList({ items, label = (value: string) => value }: { items: Array<{ label: string; count: number }>; label?: (value: string) => string }) {
  return items.length === 0 ? <div className="empty-list">Sem dados para este indicador.</div> : <div className="record-list">{items.map((item) => <article className="record-card" key={item.label}><div className="record-title-row"><strong>{label(item.label)}</strong><span className="badge">{item.count}</span></div></article>)}</div>;
}
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function statusLabel(value: string) { const labels: Record<string, string> = { active: "Ativas", weak: "Fracas", recovering: "Em recuperação", inactive: "Inativas", lost: "Perdidas", transferred: "Transferidas" }; return labels[value] || value; }
function strengthLabel(value: string) { const labels: Record<string, string> = { strong: "Fortes", medium: "Médias", weak: "Fracas", unknown: "Sem avaliação" }; return labels[value] || value; }
function productLabel(value: string) { const labels: Record<string, string> = { honey: "Mel", pollen: "Pólen", propolis: "Própolis", wax: "Cera", cerumen: "Cerume", other: "Outro" }; return labels[value] || value; }
function movementLabel(value: string) { return value === "internal_transfer" ? "Transferência interna" : value === "external_transfer" ? "Transferência externa" : "Transporte"; }
function severityLabel(value: string) { return value === "critical" ? "Crítico" : value === "attention" ? "Atenção" : "Informativo"; }
function alertTypeLabel(value: string) { const labels: Record<string, string> = { inspection_due: "Inspeção pendente", feeding_due: "Alimentação pendente", weak_colony: "Colônia fraca" }; return labels[value] || value; }
