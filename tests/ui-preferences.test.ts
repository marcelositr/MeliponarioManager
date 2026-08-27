import assert from "node:assert/strict";
import test from "node:test";
import { normalizeActiveMeliponary, normalizeTheme, readSidebarCollapsed, resolveTheme } from "../src/lib/ui-preferences.ts";

test("theme preference accepts light, dark and system and rejects unknown values", () => {
  assert.equal(normalizeTheme("light"), "light");
  assert.equal(normalizeTheme("dark"), "dark");
  assert.equal(normalizeTheme("system"), "system");
  assert.equal(normalizeTheme("hacker"), "system");
  assert.equal(normalizeTheme(null), "system");
});

test("system theme follows the operating system while explicit themes remain stable", () => {
  assert.equal(resolveTheme("system", true), "dark");
  assert.equal(resolveTheme("system", false), "light");
  assert.equal(resolveTheme("light", true), "light");
  assert.equal(resolveTheme("dark", false), "dark");
});

test("active meliponary falls back to consolidated view when the stored id is stale", () => {
  assert.equal(normalizeActiveMeliponary("m1", ["m1", "m2"]), "m1");
  assert.equal(normalizeActiveMeliponary("missing", ["m1", "m2"]), "all");
  assert.equal(normalizeActiveMeliponary("all", ["m1"]), "all");
  assert.equal(normalizeActiveMeliponary(null, ["m1"]), "all");
});

test("sidebar persistence uses only the explicit collapsed marker", () => {
  assert.equal(readSidebarCollapsed("1"), true);
  assert.equal(readSidebarCollapsed("0"), false);
  assert.equal(readSidebarCollapsed(null), false);
});
