import { save } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "react";
import type { NavigationIntent } from "../lib/navigation";
import { publicError, formatDateTimeBr } from "../lib/presentation";
import { defaultReportPeriod, formatBrl, formatReportNumber, reportFilename } from "../lib/report-presentation";
import {
  exportReportCsv,
  getAgendaReport,
  getColonyReport,
  getCostReport,
  getMeliponaryReport,
  getOperationalReport,
  getProductionReport,
  type AgendaMetrics,
  type AgendaReport,
  type ColonyReport,
  type CostReport,
  type MeliponaryReport,
  type OperationalReport,
  type ProductionAggregate,
  type ProductionReport,
  type ReportContext,
  type ReportFilter,
} from "../lib/reports-api";
import type { Colony, Meliponary, Species } from "../types";

type ReportKind = "operational" | "production" | "costs" | "agenda" | "colony" | "meliponary";
type Props = { meliponaries: Meliponary[]; colonies: Colony[]; species: Species[]; activeMeliponaryId: string; navigationIntent: NavigationIntent };

const reportOptions: Array<[ReportKind, string]> = [
  ["operational", "Visão operacional"],
  ["production", "Produção"],
  ["costs", "Custos"],
  ["agenda", "Agenda"],
  ["colony", "Histórico de colônia"],
  ["meliponary", "Meliponário"],
];

export function ReportsPage({ meliponaries, colonies, species, activeMeliponaryId, navigationIntent }: Props) {
  const initialPeriod = useMemo(() => defaultReportPeriod(), []);
  const [kind, setKind] = useState<ReportKind>(() => navigationIntent.colonyId ? "colony" : navigationIntent.meliponaryId ? "meliponary" : "operational");
  const [filter, setFilter] = useState<ReportFilter>({ ...initialPeriod, meliponaryId: activeMeliponaryId || undefined });
  const [colonyId, setColonyId] = useState(navigationIntent.colonyId || "");
  const [speciesId, setSpeciesId] = useState("");
  const [productType, setProductType] = useState("");
  const [includeAudit, setIncludeAudit] = useState(false);
  const [busy, setBusy] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [feedback, setFeedback] = useState<{ kind: "success" | "error"; text: string } | null>(null);
  const [operational, setOperational] = useState<OperationalReport | null>(null);
  const [production, setProduction] = useState<ProductionReport | null>(null);
  const [costs, setCosts] = useState<CostReport | null>(null);
  const [agenda, setAgenda] = useState<AgendaReport | null>(null);
  const [colony, setColony] = useState<ColonyReport | null>(null);
  const [meliponary, setMeliponary] = useState<MeliponaryReport | null>(null);

  useEffect(() => {
    setFilter((current) => ({ ...current, meliponaryId: activeMeliponaryId || undefined }));
  }, [activeMeliponaryId]);

  useEffect(() => {
    if (navigationIntent.view !== "reports") return;
    if (navigationIntent.colonyId) { setKind("colony"); setColonyId(navigationIntent.colonyId); }
    else if (navigationIntent.meliponaryId) setKind("meliponary");
  }, [navigationIntent]);

  const scopedColonies = filter.meliponaryId ? colonies.filter((item) => item.meliponaryId === filter.meliponaryId) : colonies;
  const hasResult = Boolean(operational || production || costs || agenda || colony || meliponary);

  async function generate() {
    if (busy) return;
    setBusy(true); setFeedback(null); clearResults();
    try {
      if (kind === "operational") setOperational(await getOperationalReport(filter));
      else if (kind === "production") setProduction(await getProductionReport({ filter, speciesId: speciesId || undefined, colonyId: colonyId || undefined, productType: productType || undefined }));
      else if (kind === "costs") setCosts(await getCostReport(filter));
      else if (kind === "agenda") setAgenda(await getAgendaReport(filter));
      else if (kind === "colony") {
        if (!colonyId) throw new Error("Selecione uma colônia para gerar o histórico.");
        setColony(await getColonyReport({ filter, colonyId, includeAudit }));
      } else {
        if (!filter.meliponaryId) throw new Error("Selecione um meliponário para gerar este relatório.");
        setMeliponary(await getMeliponaryReport(filter));
      }
    } catch (error) {
      setFeedback({ kind: "error", text: publicError(error, "Não foi possível gerar o relatório.") });
    } finally { setBusy(false); }
  }

  function clearResults() {
    setOperational(null); setProduction(null); setCosts(null); setAgenda(null); setColony(null); setMeliponary(null);
  }

  function changeKind(next: ReportKind) { setKind(next); setFeedback(null); clearResults(); }

  async function exportCsv() {
    if (exporting || !["production", "agenda", "colony", "costs"].includes(kind)) return;
    const colonyCode = colonies.find((item) => item.id === colonyId)?.code;
    const target = await save({
      title: "Exportar relatório CSV",
      defaultPath: reportFilename(kind === "costs" ? "custos-manutencao" : kind, filter.startDate, filter.endDate, kind === "colony" ? colonyCode : undefined),
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (typeof target !== "string") return;
    setExporting(true); setFeedback(null);
    try {
      const result = await exportReportCsv({ kind: kind as "production" | "agenda" | "colony" | "costs", path: target, filter, colonyId: colonyId || undefined, includeAudit, speciesId: speciesId || undefined, productType: productType || undefined });
      setFeedback({ kind: "success", text: `CSV exportado com ${result.rowCount} registro(s).` });
    } catch (error) {
      setFeedback({ kind: "error", text: publicError(error, "Não foi possível exportar o CSV.") });
    } finally { setExporting(false); }
  }

  return <div className="page-stack reports-page">
    <section className="page-heading report-no-print"><div><span className="eyebrow">Gestão</span><h1>Relatórios</h1><p>Consulte dados efetivos por período, exporte tabelas para CSV e use a impressão do sistema para papel ou PDF.</p></div></section>

    <section className="panel report-controls report-no-print" aria-label="Filtros dos relatórios">
      <div className="report-tabs" role="tablist" aria-label="Tipo de relatório">{reportOptions.map(([value, label]) => <button key={value} role="tab" aria-selected={kind === value} className={kind === value ? "report-tab active" : "report-tab"} type="button" onClick={() => changeKind(value)}>{label}</button>)}</div>
      <div className="report-filter-grid">
        <label className="field"><span>Data inicial</span><input type="date" value={filter.startDate} onChange={(event) => setFilter({ ...filter, startDate: event.target.value })} /></label>
        <label className="field"><span>Data final</span><input type="date" value={filter.endDate} onChange={(event) => setFilter({ ...filter, endDate: event.target.value })} /></label>
        <label className="field"><span>Meliponário</span><select value={filter.meliponaryId || ""} onChange={(event) => { setFilter({ ...filter, meliponaryId: event.target.value || undefined }); setColonyId(""); }}><option value="">Todos os meliponários</option>{meliponaries.map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}</select></label>
        {(kind === "production" || kind === "colony") && <label className="field"><span>Colônia{kind === "colony" ? "" : " (opcional)"}</span><select value={colonyId} onChange={(event) => setColonyId(event.target.value)}><option value="">{kind === "colony" ? "Selecione..." : "Todas as colônias"}</option>{scopedColonies.map((item) => <option value={item.id} key={item.id}>{item.code}</option>)}</select></label>}
        {kind === "production" && <><label className="field"><span>Espécie</span><select value={speciesId} onChange={(event) => setSpeciesId(event.target.value)}><option value="">Todas as espécies</option>{species.map((item) => <option value={item.id} key={item.id}>{item.commonName}</option>)}</select></label><label className="field"><span>Produto</span><select value={productType} onChange={(event) => setProductType(event.target.value)}><option value="">Todos os produtos</option><option value="honey">Mel</option><option value="pollen">Pólen</option><option value="propolis">Própolis</option><option value="wax">Cera</option><option value="cerumen">Cerume</option><option value="other">Outro</option></select></label></>}
        {kind === "colony" && <label className="check-field report-audit-toggle"><input type="checkbox" checked={includeAudit} onChange={(event) => setIncludeAudit(event.target.checked)} /><span>Histórico completo / auditoria</span></label>}
      </div>
      <div className="workspace-actions"><button type="button" onClick={() => void generate()} disabled={busy || !filter.startDate || !filter.endDate || (kind === "colony" && !colonyId) || (kind === "meliponary" && !filter.meliponaryId)}>{busy ? "Gerando…" : "Gerar relatório"}</button>{hasResult && ["production", "agenda", "colony", "costs"].includes(kind) && <button type="button" className="button-secondary" onClick={() => void exportCsv()} disabled={busy || exporting}>{exporting ? "Exportando…" : "Exportar CSV…"}</button>}{hasResult && <button type="button" className="button-secondary" onClick={() => window.print()} disabled={busy || exporting}>Imprimir…</button>}</div>
    </section>

    {feedback && <div className={`feedback-banner ${feedback.kind} report-no-print`} role={feedback.kind === "error" ? "alert" : "status"}>{feedback.text}</div>}
    {busy && <div className="empty-list" role="status">Consultando dados do relatório…</div>}

    {!busy && operational && <OperationalView report={operational} />}
    {!busy && production && <ProductionView report={production} />}
    {!busy && costs && <CostView report={costs} />}
    {!busy && agenda && <AgendaView report={agenda} />}
    {!busy && colony && <ColonyView report={colony} />}
    {!busy && meliponary && <MeliponaryView report={meliponary} />}
    {!busy && !hasResult && !feedback && <div className="empty-list report-no-print">Escolha os filtros e gere um relatório.</div>}
  </div>;
}

function ReportHeader({ title, context, subtitle }: { title: string; context: ReportContext; subtitle?: string }) {
  return <header className="report-print-header"><span className="eyebrow">MeliponarioManager</span><h1>{title}</h1>{subtitle && <p>{subtitle}</p>}<dl><div><dt>Período</dt><dd>{formatDateOnly(context.startDate)} a {formatDateOnly(context.endDate)}</dd></div><div><dt>Meliponário</dt><dd>{context.meliponaryName}</dd></div><div><dt>Gerado em</dt><dd>{formatDateTimeBr(context.generatedAt)}</dd></div></dl></header>;
}

function OperationalView({ report }: { report: OperationalReport }) {
  return <article className="report-sheet"><ReportHeader title="Visão operacional" context={report.context} subtitle="Resumo do plantel atual e dos fatos operacionais efetivos no período." /><section className="report-section"><h2>Plantel</h2><div className="report-metrics"><Metric label="Colônias" value={report.plantel.totalColonies} /><Metric label="Ativas" value={report.plantel.activeColonies} /><Metric label="Caixas ativas" value={report.plantel.activeBoxes} /><Metric label="Ocupações atuais" value={report.plantel.currentOccupancies} /></div>{report.plantel.colonyStatuses.length > 0 && <p className="report-note">Situação atual: {report.plantel.colonyStatuses.map((item) => `${item.label}: ${item.count}`).join(" · ")}</p>}</section><section className="report-section"><h2>Manejo no período</h2><div className="report-metrics"><Metric label="Inspeções" value={report.management.inspections} /><Metric label="Alimentações" value={report.management.feedings} /><Metric label="Manutenções" value={report.management.maintenance} /><Metric label="Eventos" value={report.management.events} /></div></section><ProductionSummary items={report.production} /><section className="report-section"><h2>Movimentações</h2><div className="report-metrics"><Metric label="Transferências" value={report.movements.transfers} /><Metric label="Transportes iniciados" value={report.movements.temporaryStarted} /><Metric label="Retornos concluídos" value={report.movements.returnsCompleted} /><Metric label="Abertos ao fim" value={report.movements.temporaryOpenAtEnd} /></div></section><AgendaMetricsView metrics={report.agenda} /></article>;
}

function ProductionView({ report }: { report: ProductionReport }) {
  return <article className="report-sheet"><ReportHeader title="Produção" context={report.context} /><ProductionSummary items={report.byProductUnit} />{report.rows.length === 0 ? <EmptyReport /> : <div className="table-wrap"><table className="data-table report-table"><thead><tr><th>Data</th><th>Meliponário</th><th>Colônia</th><th>Espécie</th><th>Produto</th><th>Quantidade</th><th>Observações</th></tr></thead><tbody>{report.rows.map((row) => <tr key={row.id}><td>{formatDateTimeBr(row.harvestedAt)}</td><td>{row.meliponaryName}</td><td>{row.colonyCode}</td><td>{row.speciesName}</td><td>{productLabel(row.productType)}</td><td>{formatReportNumber(row.quantity)} {row.unit}</td><td>{row.notes || row.purpose || "—"}</td></tr>)}</tbody></table></div>}</article>;
}

function CostView({ report }: { report: CostReport }) {
  return <article className="report-sheet"><ReportHeader title="Custos registrados" context={report.context} subtitle={report.sourceDescription} /><section className="report-section"><div className="report-metrics"><Metric label="Total registrado" value={formatBrl(report.total)} /></div><p className="report-note">{report.currencyAssumption}</p></section>{report.rows.length === 0 ? <EmptyReport message="Não há custos registrados neste período." /> : <div className="table-wrap"><table className="data-table report-table"><thead><tr><th>Data</th><th>Meliponário</th><th>Caixa</th><th>Colônia</th><th>Tipo</th><th>Descrição</th><th>Custo</th></tr></thead><tbody>{report.rows.map((row, index) => <tr key={`${row.maintainedAt}-${row.boxCode}-${index}`}><td>{formatDateTimeBr(row.maintainedAt)}</td><td>{row.meliponaryName}</td><td>{row.boxCode}</td><td>{row.colonyCode || "—"}</td><td>{maintenanceLabel(row.maintenanceType)}</td><td>{row.description || "—"}</td><td>{formatBrl(row.cost)}</td></tr>)}</tbody></table></div>}</article>;
}

function AgendaView({ report }: { report: AgendaReport }) {
  return <article className="report-sheet"><ReportHeader title="Agenda" context={report.context} subtitle="Cada compromisso aparece uma vez pelo seu estado final; reagendamento não é tratado automaticamente como falha." /><AgendaMetricsView metrics={report.metrics} />{report.rows.length === 0 ? <EmptyReport /> : <div className="table-wrap"><table className="data-table report-table"><thead><tr><th>Agendado</th><th>Contexto</th><th>Tarefa</th><th>Status</th><th>Conclusão</th></tr></thead><tbody>{report.rows.map((row) => <tr key={row.id}><td>{formatDateTimeBr(row.scheduledFor)}</td><td>{row.meliponaryName}{row.colonyCode ? ` · ${row.colonyCode}` : ""}</td><td>{row.title}</td><td>{taskStatusLabel(row.status)}</td><td>{row.completedAt ? `${formatDateTimeBr(row.completedAt)} · ${timingLabel(row.timing)}` : "—"}</td></tr>)}</tbody></table></div>}</article>;
}

function ColonyView({ report }: { report: ColonyReport }) {
  return <article className="report-sheet"><ReportHeader title={`Histórico da colônia ${report.identity.colonyCode}`} context={report.context} subtitle={report.includeAudit ? "Modo completo: inclui fatos anulados/revertidos e registros de auditoria relacionados." : "Modo operacional: fatos anulados e revertidos não são apresentados como válidos."} /><section className="report-section"><h2>Identificação</h2><dl className="report-details"><div><dt>Espécie</dt><dd>{report.identity.speciesName}{report.identity.scientificName ? ` · ${report.identity.scientificName}` : ""}</dd></div><div><dt>Situação</dt><dd>{report.identity.status}</dd></div><div><dt>Caixa atual</dt><dd>{report.identity.currentBoxCode || "Sem caixa"}</dd></div><div><dt>Origem</dt><dd>{report.identity.originType}{report.identity.originNotes ? ` · ${report.identity.originNotes}` : ""}</dd></div><div><dt>Colônia-mãe</dt><dd>{report.identity.motherColonyCode || "—"}</dd></div><div><dt>Fotos válidas de inspeção</dt><dd>{report.photoCount}</dd></div></dl></section>{report.timeline.length === 0 ? <EmptyReport /> : <div className="table-wrap"><table className="data-table report-table"><thead><tr><th>Data</th><th>Categoria</th><th>Fato</th><th>Detalhes</th><th>Estado</th></tr></thead><tbody>{report.timeline.map((row) => <tr key={`${row.sourceId}-${row.occurredAt}`}><td>{formatDateTimeBr(row.occurredAt)}</td><td>{row.category}</td><td>{row.title}</td><td>{row.details || "—"}</td><td>{stateLabel(row.state)}</td></tr>)}</tbody></table></div>}</article>;
}

function MeliponaryView({ report }: { report: MeliponaryReport }) {
  return <article className="report-sheet"><ReportHeader title={`Relatório do meliponário ${report.context.meliponaryName}`} context={report.context} subtitle="Consolidação por período, distinta do Dashboard de situação imediata." /><OperationalView report={report.operational} /><section className="report-section"><h2>Custos de manutenção registrados</h2><div className="report-metrics"><Metric label="Total" value={formatBrl(report.maintenanceCostTotal)} /></div></section></article>;
}

function ProductionSummary({ items }: { items: ProductionAggregate[] }) { return <section className="report-section"><h2>Produção por produto e unidade</h2>{items.length === 0 ? <p className="report-note">Nenhuma produção efetiva no período.</p> : <div className="report-metrics">{items.map((item) => <Metric key={`${item.groupLabel}-${item.productType}-${item.unit}`} label={item.groupLabel} value={`${formatReportNumber(item.quantity)} ${item.unit}`} />)}</div>}</section>; }
function AgendaMetricsView({ metrics }: { metrics: AgendaMetrics }) { return <section className="report-section"><h2>Agenda</h2><div className="report-metrics"><Metric label="Criadas" value={metrics.created} /><Metric label="Agendadas no período" value={metrics.scheduled} /><Metric label="Concluídas" value={metrics.completed} /><Metric label="No prazo" value={metrics.completedOnTime} /><Metric label="Após o prazo" value={metrics.completedLate} /><Metric label="Canceladas" value={metrics.cancelled} /><Metric label="Ignoradas" value={metrics.skipped} /><Metric label="Reagendadas" value={metrics.rescheduled} /><Metric label="Pendentes atrasadas" value={metrics.overduePending} /></div></section>; }
function Metric({ label, value }: { label: string; value: string | number }) { return <div className="report-metric"><span>{label}</span><strong>{value}</strong></div>; }
function EmptyReport({ message = "Nenhum registro encontrado para o período e filtros selecionados." }: { message?: string }) { return <div className="empty-list">{message}</div>; }
function formatDateOnly(value: string) { const [year, month, day] = value.split("-"); return `${day}/${month}/${year}`; }
function productLabel(value: string) { return ({ honey: "Mel", pollen: "Pólen", propolis: "Própolis", wax: "Cera", cerumen: "Cerume", other: "Outro produto" } as Record<string, string>)[value] || value; }
function taskStatusLabel(value: string) { return ({ pending: "Pendente", completed: "Concluída", cancelled: "Cancelada", skipped: "Ignorada", rescheduled: "Reagendada" } as Record<string, string>)[value] || value; }
function timingLabel(value: string) { return value === "on_time" ? "No prazo" : value === "late" ? "Após o prazo" : "Não se aplica"; }
function stateLabel(value: string) { return ({ effective: "Efetivo", corrected: "Corrigido", voided: "Anulado", reversed: "Revertido", audit: "Auditoria" } as Record<string, string>)[value] || value; }
function maintenanceLabel(value: string) { return ({ cleaning: "Limpeza", repair: "Reparo", painting: "Pintura", waterproofing: "Impermeabilização", roof: "Cobertura", entrance: "Entrada", internal_structure: "Estrutura interna", inspection: "Revisão da caixa", other: "Outro" } as Record<string, string>)[value] || value; }
