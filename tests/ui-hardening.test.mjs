import assert from "node:assert/strict";
import test from "node:test";
import { toNavigationIntent } from "../src/lib/navigation.ts";
import { formatDateTimeBr, linkedFactLabel, publicError } from "../src/lib/presentation.ts";

test("contextual navigation preserves task and entity intent", () => {
  assert.deepEqual(toNavigationIntent("agenda"), { view: "agenda" });
  const intent = {
    view: "agenda",
    taskId: "task-42",
    colonyId: "colony-7",
    meliponaryId: "mel-2",
    action: "open",
  };
  assert.deepEqual(toNavigationIntent(intent), intent);
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
