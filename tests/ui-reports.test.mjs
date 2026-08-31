import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { defaultReportPeriod, reportFilename, sanitizeReportFilename } from "../src/lib/report-presentation.ts";

const reports = fs.readFileSync(new URL("../src/pages/ReportsPage.tsx", import.meta.url), "utf8");
const reportViews = fs.readFileSync(new URL("../src/pages/reports/ReportViews.tsx", import.meta.url), "utf8");
const menu = fs.readFileSync(new URL("../src/components/TopMenu.tsx", import.meta.url), "utf8");
const sidebar = fs.readFileSync(new URL("../src/components/Sidebar.tsx", import.meta.url), "utf8");
const router = fs.readFileSync(new URL("../src/components/WorkspaceRouter.tsx", import.meta.url), "utf8");
const printCss = fs.readFileSync(new URL("../src/styles/reports.css", import.meta.url), "utf8");
const dataPage = fs.readFileSync(new URL("../src/pages/DataManagementPage.tsx", import.meta.url), "utf8");

test("report period defaults to first day through current local day", () => {
  assert.deepEqual(defaultReportPeriod(new Date(2026, 7, 29, 15, 30)), { startDate: "2026-08-01", endDate: "2026-08-29" });
});

test("report filenames are predictable and safe for desktop filesystems", () => {
  assert.equal(sanitizeReportFilename('Colônia JAT/001: "A"'), "colonia-jat-001-a");
  assert.equal(reportFilename("producao", "2026-08-01", "2026-08-31"), "producao-2026-08-01-a-2026-08-31.csv");
});

test("reports workspace exposes global filters and active meliponary default", () => {
  assert.match(reports, /Data inicial/);
  assert.match(reports, /Data final/);
  assert.match(reports, /Todos os meliponários/);
  assert.match(reports, /activeMeliponaryId \|\| undefined/);
});

test("report switching, empty state, loading and csv controls are explicit", () => {
  assert.match(reports, /Visão operacional/);
  assert.match(reports, /Histórico de colônia/);
  assert.match(reports, /Consultando dados do relatório/);
  assert.match(reportViews, /Nenhum registro encontrado para o período e filtros selecionados/);
  assert.match(reports, /Exportar CSV…/);
});

test("report orchestration stays separate from report rendering views", () => {
  assert.match(reports, /from "\.\/reports\/ReportViews"/);
  assert.doesNotMatch(reports, /function ReportHeader/);
  for (const view of ["OperationalView", "ProductionView", "CostView", "AgendaView", "ColonyView", "MeliponaryView"]) {
    assert.match(reportViews, new RegExp(`export function ${view}`));
  }
});

test("contextual navigation can open colony or meliponary reports", () => {
  assert.match(reports, /navigationIntent\.colonyId \? "colony"/);
  assert.match(reports, /navigationIntent\.meliponaryId \? "meliponary"/);
  assert.match(router, /activeView === "reports"/);
});

test("file menu and sidebar lead to reports while data remains administrative", () => {
  assert.match(menu, /onNavigate\("reports"\)/);
  assert.match(sidebar, /view: "reports", label: "Relatórios"/);
  assert.match(sidebar, /view: "data", label: "Dados"/);
  assert.doesNotMatch(dataPage, /generateManagementReport/);
});

test("print mode removes application controls and forces paper-friendly output", () => {
  assert.match(reports, /window\.print\(\)/);
  assert.match(printCss, /@media print/);
  assert.match(printCss, /\.menu-bar, \.sidebar, \.context-bar, \.status-bar/);
  assert.match(printCss, /background: #fff !important/);
  assert.match(printCss, /button, input, select, textarea \{ display: none !important; \}/);
});
