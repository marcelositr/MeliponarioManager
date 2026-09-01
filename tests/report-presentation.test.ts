import assert from "node:assert/strict";
import test from "node:test";
import { nextReportTabIndex } from "../src/lib/report-presentation.ts";

test("report tabs wrap with horizontal arrow keys", () => {
  assert.equal(nextReportTabIndex(0, "ArrowRight", 6), 1);
  assert.equal(nextReportTabIndex(5, "ArrowRight", 6), 0);
  assert.equal(nextReportTabIndex(5, "ArrowLeft", 6), 4);
  assert.equal(nextReportTabIndex(0, "ArrowLeft", 6), 5);
});

test("report tabs support Home and End", () => {
  assert.equal(nextReportTabIndex(3, "Home", 6), 0);
  assert.equal(nextReportTabIndex(2, "End", 6), 5);
});

test("report tab navigation ignores unrelated or invalid input", () => {
  assert.equal(nextReportTabIndex(2, "Tab", 6), null);
  assert.equal(nextReportTabIndex(-1, "ArrowRight", 6), null);
  assert.equal(nextReportTabIndex(6, "ArrowLeft", 6), null);
  assert.equal(nextReportTabIndex(0, "ArrowRight", 0), null);
});
