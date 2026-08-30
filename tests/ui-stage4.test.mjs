import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

async function source(path) {
  return readFile(new URL(`../${path}`, import.meta.url), "utf8");
}

test("Agenda is a first-class shell route and receives the active meliponary context", async () => {
  const [types, sidebar, router, app] = await Promise.all([
    source("src/types.ts"),
    source("src/components/Sidebar.tsx"),
    source("src/components/WorkspaceRouter.tsx"),
    source("src/App.tsx"),
  ]);
  assert.match(types, /\| "agenda"/);
  assert.match(sidebar, /view: "agenda", label: "Agenda"/);
  assert.match(router, /activeView === "agenda"/);
  assert.match(router, /<AgendaWorkspacePage/);
  assert.match(app, /scopedMeliponaryId/);
  assert.match(app, /activeMeliponaryId=\{scopedMeliponaryId\}/);
});

test("Agenda keeps the complete operational action set and contextual query", async () => {
  const agenda = await source("src/pages/AgendaPage.tsx");
  for (const action of ["createTask", "rescheduleTask", "cancelTask", "skipTask", "duplicateTask", "completeGenericTask", "completeInspectionTask", "completeFeedingTask", "completeMaintenanceTask"]) {
    assert.match(agenda, new RegExp(action));
  }
  assert.match(agenda, /meliponaryId: activeMeliponaryId \|\| undefined/);
  assert.match(agenda, /getAgendaSummary\(activeMeliponaryId \|\| undefined\)/);
});

test("alerts and dashboard expose actionable Agenda-aware navigation", async () => {
  const [types, alerts, dashboard, navigation] = await Promise.all([
    source("src/types.ts"),
    source("src/pages/AlertsPage.tsx"),
    source("src/pages/DashboardPage.tsx"),
    source("src/lib/navigation.ts"),
  ]);
  assert.match(types, /recommendedAction: string/);
  assert.match(types, /taskId\?: string \| null/);
  assert.match(alerts, /item\.meliponaryId === activeMeliponaryId/);
  assert.match(alerts, /taskId: item\.taskId/);
  assert.match(alerts, /recommendedIntent\(item\)/);
  assert.match(dashboard, /getAgendaSummary/);
  assert.match(dashboard, /taskId: alert\.taskId/);
  assert.match(navigation, /type NavigationIntent/);
});

test("record centers consume the dedicated Stage 4 backend projections", async () => {
  const [centers, colonies, boxes, meliponaries] = await Promise.all([
    source("src/components/OperationalRecordCenter.tsx"),
    source("src/pages/ColoniesPage.tsx"),
    source("src/pages/BoxesPage.tsx"),
    source("src/pages/MeliponariesPage.tsx"),
  ]);
  for (const contract of ["getColonyRecordCenter", "getBoxRecordCenter", "getMeliponaryRecordCenter", "listBoxOccupancies", "listBoxContextPhotos", "listTasks"]) {
    assert.match(centers, new RegExp(contract));
  }
  assert.match(colonies, /<ColonyOperationalCenter/);
  assert.match(boxes, /<BoxOperationalCenter/);
  assert.match(meliponaries, /<MeliponaryOperationalCenter/);
});
