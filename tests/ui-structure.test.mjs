import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

async function source(path) {
  return readFile(new URL(`../${path}`, import.meta.url), "utf8");
}

test("sidebar preserves enterprise navigation groups and keeps photos out of the primary menu", async () => {
  const sidebar = await source("src/components/Sidebar.tsx");
  for (const group of ["Operação", "Plantel", "Manejo", "Rastreabilidade", "Administração"]) {
    assert.match(sidebar, new RegExp(group));
  }
  assert.doesNotMatch(sidebar, /label: "Fotos"/);
});

test("dialog supports Escape and remains modal", async () => {
  const dialog = await source("src/components/Dialog.tsx");
  assert.match(dialog, /event\.key === "Escape"/);
  assert.match(dialog, /aria-modal="true"/);
  assert.match(dialog, /role="dialog"/);
});

test("shell exposes active meliponary context and the persistent status bar", async () => {
  const app = await source("src/App.tsx");
  assert.match(app, /Todos os meliponários/);
  assert.match(app, /meliponary-selector/);
  assert.match(app, /<StatusBar/);
});
