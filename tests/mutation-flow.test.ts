import assert from "node:assert/strict";
import test from "node:test";
import { runMutationFlow } from "../src/lib/mutation-flow.ts";

test("successful mutation and refresh report success", async () => {
  const calls: string[] = [];
  const result = await runMutationFlow(
    async () => { calls.push("mutation"); },
    async () => { calls.push("refresh"); },
  );

  assert.deepEqual(calls, ["mutation", "refresh"]);
  assert.deepEqual(result, { status: "success" });
});

test("mutation failure does not attempt refresh", async () => {
  const failure = new Error("mutation failed");
  let refreshCalls = 0;
  const result = await runMutationFlow(
    async () => { throw failure; },
    async () => { refreshCalls += 1; },
  );

  assert.equal(refreshCalls, 0);
  assert.equal(result.status, "mutation-failed");
  if (result.status === "mutation-failed") assert.equal(result.error, failure);
});

test("refresh failure preserves successful mutation outcome", async () => {
  const refreshFailure = new Error("refresh failed");
  let mutationCalls = 0;
  const result = await runMutationFlow(
    async () => { mutationCalls += 1; },
    async () => { throw refreshFailure; },
  );

  assert.equal(mutationCalls, 1);
  assert.equal(result.status, "refresh-failed");
  if (result.status === "refresh-failed") assert.equal(result.error, refreshFailure);
});
