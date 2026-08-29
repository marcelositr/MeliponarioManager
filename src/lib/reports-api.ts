import { invoke } from "@tauri-apps/api/core";

export type ReportFilter = { startDate: string; endDate: string; meliponaryId?: string };
export type ReportContext = { startDate: string; endDate: string; generatedAt: string; meliponaryId?: string | null; meliponaryName: string };
export type CountByLabel = { key: string; label: string; count: number };
export type ProductionAggregate = { groupLabel: string; productType: string; unit: string; quantity: number };
export type AgendaMetrics = { created: number; scheduled: number; completed: number; completedOnTime: number; completedLate: number; cancelled: number; skipped: number; rescheduled: number; overduePending: number };
export type OperationalReport = {
  context: ReportContext;
  plantel: { totalColonies: number; activeColonies: number; activeBoxes: number; currentOccupancies: number; colonyStatuses: CountByLabel[] };
  management: { inspections: number; feedings: number; maintenance: number; events: number };
  production: ProductionAggregate[];
  movements: { transfers: number; temporaryStarted: number; returnsCompleted: number; temporaryOpenAtEnd: number };
  agenda: AgendaMetrics;
};
export type ProductionReportRow = { id: string; harvestedAt: string; meliponaryId: string; meliponaryName: string; colonyId: string; colonyCode: string; speciesId: string; speciesName: string; productType: string; quantity: number; unit: string; purpose?: string | null; notes?: string | null };
export type ProductionReport = { context: ReportContext; rows: ProductionReportRow[]; byProductUnit: ProductionAggregate[]; byColony: ProductionAggregate[]; byMeliponary: ProductionAggregate[]; bySpecies: ProductionAggregate[] };
export type CostReportRow = { maintainedAt: string; meliponaryName: string; boxCode: string; colonyCode?: string | null; maintenanceType: string; performedBy?: string | null; description?: string | null; cost: number };
export type CostReport = { context: ReportContext; sourceDescription: string; currencyAssumption: string; total: number; rows: CostReportRow[] };
export type AgendaReportRow = { id: string; scheduledFor: string; meliponaryName: string; colonyCode?: string | null; boxCode?: string | null; taskType: string; title: string; status: string; completedAt?: string | null; timing: string; rescheduledFromId?: string | null };
export type AgendaReport = { context: ReportContext; metrics: AgendaMetrics; rows: AgendaReportRow[] };
export type ColonyHistoryRow = { sourceId: string; occurredAt: string; category: string; title: string; details?: string | null; state: string };
export type ColonyReport = { context: ReportContext; identity: { colonyId: string; colonyCode: string; meliponaryName: string; speciesName: string; scientificName?: string | null; originType: string; originNotes?: string | null; installedAt?: string | null; status: string; currentBoxCode?: string | null; motherColonyCode?: string | null }; includeAudit: boolean; photoCount: number; timeline: ColonyHistoryRow[] };
export type MeliponaryReport = { context: ReportContext; operational: OperationalReport; maintenanceCostTotal: number };
export type CsvExportResult = { path: string; rowCount: number };

export function getOperationalReport(filter: ReportFilter) { return invoke<OperationalReport>("get_operational_report", { filter }); }
export function getProductionReport(input: { filter: ReportFilter; speciesId?: string; colonyId?: string; productType?: string }) { return invoke<ProductionReport>("get_production_report", { input }); }
export function getCostReport(filter: ReportFilter) { return invoke<CostReport>("get_cost_report", { filter }); }
export function getAgendaReport(filter: ReportFilter) { return invoke<AgendaReport>("get_agenda_report", { filter }); }
export function getColonyReport(input: { filter: ReportFilter; colonyId: string; includeAudit: boolean }) { return invoke<ColonyReport>("get_colony_report", { input }); }
export function getMeliponaryReport(filter: ReportFilter) { return invoke<MeliponaryReport>("get_meliponary_report", { filter }); }
export function exportReportCsv(input: { kind: "production" | "agenda" | "colony" | "costs"; path: string; filter: ReportFilter; colonyId?: string; includeAudit?: boolean; speciesId?: string; productType?: string }) { return invoke<CsvExportResult>("export_report_csv", { input }); }
