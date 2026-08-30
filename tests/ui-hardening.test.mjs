import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { reconcileManualMeliponaryChange, toNavigationIntent } from "../src/lib/navigation.ts";
import { formatDateTimeBr, linkedFactLabel, publicError } from "../src/lib/presentation.ts";

async function source(path) {
  return readFile(new URL(`../${path}`, import.meta.url), "utf8");
}

test("contextual navigation preserves deep links but manual meliponary changes clear stale entity intent", () => {
  assert.deepEqual(toNavigationIntent("agenda"), { view: "agenda" });
  const deepLink = {
    view: "colonies",
    colonyId: "colony-a",
    meliponaryId: "mel-a",
    action: "open",
  };
  assert.deepEqual(toNavigationIntent(deepLink), deepLink);

  const afterManualSwitch = reconcileManualMeliponaryChange(deepLink);
  assert.deepEqual(afterManualSwitch, { view: "colonies" });

  const colonies = [
    { id: "colony-a", meliponaryId: "mel-a" },
    { id: "colony-b", meliponaryId: "mel-b" },
  ];
  const scopedToB = colonies.filter((item) => item.meliponaryId === "mel-b");
  const visibleAfterSwitch = afterManualSwitch.colonyId
    ? scopedToB.filter((item) => item.id === afterManualSwitch.colonyId)
    : scopedToB;
  assert.deepEqual(visibleAfterSwitch.map((item) => item.id), ["colony-b"]);
});

test("shell distinguishes automatic deep-link context changes from manual selector changes", async () => {
  const app = await source("src/App.tsx");
  assert.match(app, /changeMeliponary\(next\.meliponaryId, "navigation"\)/);
  assert.match(app, /changeMeliponary\(event\.target\.value, "manual"\)/);
  assert.match(app, /reconcileManualMeliponaryChange/);
});

test("dialog lifecycle does not restart autofocus when parent callbacks are recreated", async () => {
  const dialog = await source("src/components/Dialog.tsx");
  assert.match(dialog, /const onCloseRef = useRef\(onClose\)/);
  assert.match(dialog, /onCloseRef\.current = onClose/);
  assert.match(dialog, /onCloseRef\.current\(\)/);
  assert.match(dialog, /}, \[open\]\);/);
  assert.doesNotMatch(dialog, /\[open,\s*onClose\]/);
  assert.match(dialog, /event\.key !== "Tab"\) return;/);
  assert.match(dialog, /isTopDialog/);
  assert.match(dialog, /previousFocus\.focus\(\)/);
});

test("dialog has a structural footer outside the scrollable body and runtime styles", async () => {
  const [dialog, css, app, main] = await Promise.all([
    source("src/components/Dialog.tsx"),
    source("src/styles.css"),
    source("src/App.tsx"),
    source("src/main.tsx"),
  ]);
  assert.match(dialog, /<div className="dialog-body">\{body\}<\/div>\s*\{footerContent && <footer className="dialog-footer">/);
  assert.match(dialog, /splitDialogActions/);
  assert.match(dialog, /form \? \{ form \}/);
  assert.match(css, /\.dialog-footer\s*\{/);
  assert.match(css, /\.dialog-body\s*\{[^}]*min-height:\s*0;[^}]*overflow:\s*auto;/s);
  assert.match(main, /import "\.\/styles\.css"/);
  assert.doesNotMatch(app, /hardening\.css/);
});

test("shared action groups provide predictable spacing and wrapping", async () => {
  const [css, agenda] = await Promise.all([
    source("src/styles.css"),
    source("src/pages/AgendaWorkspacePage.tsx"),
  ]);
  assert.match(css, /\.workspace-actions, \.form-actions, \.dialog-actions, \.quick-actions, \.page-toolbar-controls, \.record-actions/);
  assert.match(css, /gap:\s*var\(--space-2\);\s*flex-wrap:\s*wrap;/);
  assert.match(agenda, /className="workspace-actions"/);
});

test("record action menu is a viewport-aware portal rather than table-flow content", async () => {
  const [actions, css] = await Promise.all([
    source("src/components/RecordActions.tsx"),
    source("src/styles.css"),
  ]);
  assert.match(actions, /createPortal/);
  assert.match(actions, /getBoundingClientRect\(\)/);
  assert.match(actions, /window\.addEventListener\("scroll", reposition, true\)/);
  assert.match(actions, /window\.addEventListener\("resize", reposition\)/);
  assert.match(actions, /role="menu"/);
  assert.match(actions, /role="menuitem"/);
  assert.match(css, /\.action-menu-popover\s*\{[^}]*position:\s*fixed;/s);
  assert.match(css, /max-width:\s*min\(260px, calc\(100vw - 16px\)\)/);
});

test("theme tokens define foreground contracts and reports use canonical surfaces", async () => {
  const [css, enterprise, reports] = await Promise.all([
    source("src/styles.css"),
    source("src/styles/enterprise.css"),
    source("src/styles/reports.css"),
  ]);
  assert.match(css, /--on-primary:\s*#[0-9a-fA-F]{6}/);
  assert.match(css, /--on-danger:\s*#[0-9a-fA-F]{6}/);
  assert.match(css, /color:\s*var\(--on-primary\);\s*background:\s*var\(--primary\)/);
  assert.match(enterprise, /button\.button-danger[^}]*color:\s*var\(--on-danger\)/s);
  assert.doesNotMatch(reports, /--border-subtle|--surface-panel|--surface-subtle/);
  assert.match(reports, /background:\s*var\(--surface-raised\)/);
});

test("desktop selection contract protects chrome while preserving editable and selectable content", async () => {
  const css = await source("src/styles.css");
  assert.match(css, /\.application-shell[^}]*user-select:\s*none;/s);
  assert.match(css, /input, textarea, \[contenteditable="true"\], \.selectable, \.selectable \* \{ user-select:\s*text; \}/);
  assert.doesNotMatch(css, /\*\s*\{[^}]*user-select:\s*none/s);
});

test("responsive contract no longer depends on a 900px root floor", async () => {
  const [css, app, tauriRaw] = await Promise.all([
    source("src/styles.css"),
    source("src/App.tsx"),
    source("src-tauri/tauri.conf.json"),
  ]);
  const tauri = JSON.parse(tauriRaw);
  const windowConfig = tauri.app.windows[0];
  assert.doesNotMatch(css, /min-width:\s*900px/);
  assert.match(css, /@media \(max-width:\s*1199px\)/);
  assert.match(css, /@media \(max-width:\s*899px\)/);
  assert.match(css, /@media \(max-height:\s*700px\)/);
  assert.match(css, /\.workspace-content[^}]*overflow-x:\s*hidden/s);
  assert.match(css, /\.table-wrap[^}]*overflow:\s*auto/s);
  assert.match(app, /COMPACT_VIEWPORT_WIDTH = 900/);
  assert.equal(windowConfig.minWidth, 760);
  assert.equal(windowConfig.minHeight, 520);
  assert.equal(tauri.version, "0.8.0");
});

test("theme choices use menuitemradio while ordinary commands remain menuitem", async () => {
  const menu = await source("src/components/TopMenu.tsx");
  assert.match(menu, /label="Tema claro" radio checked=/);
  assert.match(menu, /label="Tema escuro" radio checked=/);
  assert.match(menu, /label="Seguir sistema" radio checked=/);
  assert.match(menu, /role=\{radio \? "menuitemradio" : "menuitem"\}/);
  assert.match(menu, /aria-checked=\{radio \? checked : undefined\}/);
  assert.match(menu, /label="Atualizar"/);
});

test("temporary transport exposes open, complete and reopen lifecycle in the movements UI", async () => {
  const [page, api] = await Promise.all([
    source("src/pages/MovementsPage.tsx"),
    source("src/lib/transport-api.ts"),
  ]);
  assert.match(page, /Transporte aberto/);
  assert.match(page, /Registrar retorno…/);
  assert.match(page, /Reabrir transporte…/);
  assert.match(page, /Concluir transporte/);
  assert.match(api, /complete_transport/);
  assert.match(api, /list_transport_returns/);
  assert.match(api, /reopen_transport/);
});

test("temporary transport UI does not offer reopening while another transport is open", async () => {
  const page = await source("src/pages/MovementsPage.tsx");
  assert.match(page, /const hasOpenTransport = useMemo/);
  assert.match(page, /if \(transportReturn\) \{\s*if \(!hasOpenTransport\) secondary\.push\(\{ label: "Reabrir transporte…"/);
  assert.match(page, /movementForm\.movementType === "transport" && hasOpenTransport/);
});

test("presentation keeps canonical persistence out of visible dates", () => {
  assert.equal(formatDateTimeBr("2026-08-28 21:37:42"), "28/08/2026 21:37");
  assert.equal(formatDateTimeBr("2026-08-28T09:05:00"), "28/08/2026 09:05");
  assert.equal(formatDateTimeBr(undefined), "—");
});

test("technical backend errors are replaced with a public fallback", () => {
  const fallback = "Não foi possível salvar.";
  assert.equal(publicError("SQLx database error: UNIQUE constraint failed", fallback), fallback);
  assert.equal(publicError(new Error("SQLite constraint failed"), fallback), fallback);
  assert.equal(publicError("A colônia já ocupa outra caixa.", fallback), "A colônia já ocupa outra caixa.");
});

test("linked facts expose human meaning instead of internal identifiers", () => {
  assert.equal(linkedFactLabel("inspection"), "Inspeção vinculada");
  assert.equal(linkedFactLabel("feeding"), "Alimentação vinculada");
  assert.equal(linkedFactLabel("unknown_internal_type"), "Registro operacional vinculado");
});
