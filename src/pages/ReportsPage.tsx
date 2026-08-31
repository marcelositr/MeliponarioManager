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
import { AgendaView, ColonyView, CostView, MeliponaryView, OperationalView, ProductionView } from "./reports/ReportViews";

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
