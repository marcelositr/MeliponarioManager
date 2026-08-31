import { formatDateTimeBr } from "../../lib/presentation";
import { formatBrl, formatReportNumber } from "../../lib/report-presentation";
import type {
  AgendaMetrics,
  AgendaReport,
  ColonyReport,
  CostReport,
  MeliponaryReport,
  OperationalReport,
  ProductionAggregate,
  ProductionReport,
  ReportContext,
} from "../../lib/reports-api";

function ReportHeader({ title, context, subtitle }: { title: string; context: ReportContext; subtitle?: string }) {
  return <header className="report-print-header"><span className="eyebrow">MeliponarioManager</span><h1>{title}</h1>{subtitle && <p>{subtitle}</p>}<dl><div><dt>Período</dt><dd>{formatDateOnly(context.startDate)} a {formatDateOnly(context.endDate)}</dd></div><div><dt>Meliponário</dt><dd>{context.meliponaryName}</dd></div><div><dt>Gerado em</dt><dd>{formatDateTimeBr(context.generatedAt)}</dd></div></dl></header>;
}

export function OperationalView({ report }: { report: OperationalReport }) {
  return <article className="report-sheet"><ReportHeader title="Visão operacional" context={report.context} subtitle="Resumo do plantel atual e dos fatos operacionais efetivos no período." /><section className="report-section"><h2>Plantel</h2><div className="report-metrics"><Metric label="Colônias" value={report.plantel.totalColonies} /><Metric label="Ativas" value={report.plantel.activeColonies} /><Metric label="Caixas ativas" value={report.plantel.activeBoxes} /><Metric label="Ocupações atuais" value={report.plantel.currentOccupancies} /></div>{report.plantel.colonyStatuses.length > 0 && <p className="report-note">Situação atual: {report.plantel.colonyStatuses.map((item) => `${item.label}: ${item.count}`).join(" · ")}</p>}</section><section className="report-section"><h2>Manejo no período</h2><div className="report-metrics"><Metric label="Inspeções" value={report.management.inspections} /><Metric label="Alimentações" value={report.management.feedings} /><Metric label="Manutenções" value={report.management.maintenance} /><Metric label="Eventos" value={report.management.events} /></div></section><ProductionSummary items={report.production} /><section className="report-section"><h2>Movimentações</h2><div className="report-metrics"><Metric label="Transferências" value={report.movements.transfers} /><Metric label="Transportes iniciados" value={report.movements.temporaryStarted} /><Metric label="Retornos concluídos" value={report.movements.returnsCompleted} /><Metric label="Abertos ao fim" value={report.movements.temporaryOpenAtEnd} /></div></section><AgendaMetricsView metrics={report.agenda} /></article>;
}

export function ProductionView({ report }: { report: ProductionReport }) {
  return <article className="report-sheet"><ReportHeader title="Produção" context={report.context} /><ProductionSummary items={report.byProductUnit} />{report.rows.length === 0 ? <EmptyReport /> : <div className="table-wrap"><table className="data-table report-table"><thead><tr><th>Data</th><th>Meliponário</th><th>Colônia</th><th>Espécie</th><th>Produto</th><th>Quantidade</th><th>Observações</th></tr></thead><tbody>{report.rows.map((row) => <tr key={row.id}><td>{formatDateTimeBr(row.harvestedAt)}</td><td>{row.meliponaryName}</td><td>{row.colonyCode}</td><td>{row.speciesName}</td><td>{productLabel(row.productType)}</td><td>{formatReportNumber(row.quantity)} {row.unit}</td><td>{row.notes || row.purpose || "—"}</td></tr>)}</tbody></table></div>}</article>;
}

export function CostView({ report }: { report: CostReport }) {
  return <article className="report-sheet"><ReportHeader title="Custos registrados" context={report.context} subtitle={report.sourceDescription} /><section className="report-section"><div className="report-metrics"><Metric label="Total registrado" value={formatBrl(report.total)} /></div><p className="report-note">{report.currencyAssumption}</p></section>{report.rows.length === 0 ? <EmptyReport message="Não há custos registrados neste período." /> : <div className="table-wrap"><table className="data-table report-table"><thead><tr><th>Data</th><th>Meliponário</th><th>Caixa</th><th>Colônia</th><th>Tipo</th><th>Descrição</th><th>Custo</th></tr></thead><tbody>{report.rows.map((row, index) => <tr key={`${row.maintainedAt}-${row.boxCode}-${index}`}><td>{formatDateTimeBr(row.maintainedAt)}</td><td>{row.meliponaryName}</td><td>{row.boxCode}</td><td>{row.colonyCode || "—"}</td><td>{maintenanceLabel(row.maintenanceType)}</td><td>{row.description || "—"}</td><td>{formatBrl(row.cost)}</td></tr>)}</tbody></table></div>}</article>;
}

export function AgendaView({ report }: { report: AgendaReport }) {
  return <article className="report-sheet"><ReportHeader title="Agenda" context={report.context} subtitle="Cada compromisso aparece uma vez pelo seu estado final; reagendamento não é tratado automaticamente como falha." /><AgendaMetricsView metrics={report.metrics} />{report.rows.length === 0 ? <EmptyReport /> : <div className="table-wrap"><table className="data-table report-table"><thead><tr><th>Agendado</th><th>Contexto</th><th>Tarefa</th><th>Status</th><th>Conclusão</th></tr></thead><tbody>{report.rows.map((row) => <tr key={row.id}><td>{formatDateTimeBr(row.scheduledFor)}</td><td>{row.meliponaryName}{row.colonyCode ? ` · ${row.colonyCode}` : ""}</td><td>{row.title}</td><td>{taskStatusLabel(row.status)}</td><td>{row.completedAt ? `${formatDateTimeBr(row.completedAt)} · ${timingLabel(row.timing)}` : "—"}</td></tr>)}</tbody></table></div>}</article>;
}

export function ColonyView({ report }: { report: ColonyReport }) {
  return <article className="report-sheet"><ReportHeader title={`Histórico da colônia ${report.identity.colonyCode}`} context={report.context} subtitle={report.includeAudit ? "Modo completo: inclui fatos anulados/revertidos e registros de auditoria relacionados." : "Modo operacional: fatos anulados e revertidos não são apresentados como válidos."} /><section className="report-section"><h2>Identificação</h2><dl className="report-details"><div><dt>Espécie</dt><dd>{report.identity.speciesName}{report.identity.scientificName ? ` · ${report.identity.scientificName}` : ""}</dd></div><div><dt>Situação</dt><dd>{report.identity.status}</dd></div><div><dt>Caixa atual</dt><dd>{report.identity.currentBoxCode || "Sem caixa"}</dd></div><div><dt>Origem</dt><dd>{report.identity.originType}{report.identity.originNotes ? ` · ${report.identity.originNotes}` : ""}</dd></div><div><dt>Colônia-mãe</dt><dd>{report.identity.motherColonyCode || "—"}</dd></div><div><dt>Fotos válidas de inspeção</dt><dd>{report.photoCount}</dd></div></dl></section>{report.timeline.length === 0 ? <EmptyReport /> : <div className="table-wrap"><table className="data-table report-table"><thead><tr><th>Data</th><th>Categoria</th><th>Fato</th><th>Detalhes</th><th>Estado</th></tr></thead><tbody>{report.timeline.map((row) => <tr key={`${row.sourceId}-${row.occurredAt}`}><td>{formatDateTimeBr(row.occurredAt)}</td><td>{row.category}</td><td>{row.title}</td><td>{row.details || "—"}</td><td>{stateLabel(row.state)}</td></tr>)}</tbody></table></div>}</article>;
}

export function MeliponaryView({ report }: { report: MeliponaryReport }) {
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
