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

test("dialog has a structural footer outside the scrollable body", async () => {
  const [dialog, css] = await Promise.all([
    source("src/components/Dialog.tsx"),
    source("src/hardening.css"),
  ]);
  assert.match(dialog, /<div className="dialog-body">\{body\}<\/div>\s*\{footerContent && <footer className="dialog-footer">/);
  assert.match(dialog, /splitDialogActions/);
  assert.match(dialog, /form \? \{ form \}/);
  assert.match(css, /\.dialog-footer\s*\{/);
  assert.doesNotMatch(css, /dialog-body[\s\S]{0,300}position:\s*sticky/);
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
