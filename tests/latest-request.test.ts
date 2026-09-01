import assert from "node:assert/strict";
import test from "node:test";
import { createLatestRequestController, runLatestRequest } from "../src/lib/latest-request.ts";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((nextResolve, nextReject) => {
    resolve = nextResolve;
    reject = nextReject;
  });
  return { promise, resolve, reject };
}

test("newer request wins when an older request resolves later", async () => {
  const controller = createLatestRequestController();
  const first = deferred<string>();
  const second = deferred<string>();
  const applied: string[] = [];

  const firstRun = runLatestRequest(controller, () => first.promise, {
    onSuccess: (value) => applied.push(value),
  });
  const secondRun = runLatestRequest(controller, () => second.promise, {
    onSuccess: (value) => applied.push(value),
  });

  second.resolve("new");
  assert.equal(await secondRun, "success");
  first.resolve("old");
  assert.equal(await firstRun, "stale");
  assert.deepEqual(applied, ["new"]);
});

test("stale rejection is ignored while current rejection is reported", async () => {
  const controller = createLatestRequestController();
  const first = deferred<string>();
  const second = deferred<string>();
  const errors: unknown[] = [];

  const firstRun = runLatestRequest(controller, () => first.promise, {
    onSuccess: () => undefined,
    onError: (error) => errors.push(error),
  });
  const secondRun = runLatestRequest(controller, () => second.promise, {
    onSuccess: () => undefined,
    onError: (error) => errors.push(error),
  });

  first.reject(new Error("old failure"));
  assert.equal(await firstRun, "stale");
  const currentError = new Error("current failure");
  second.reject(currentError);
  assert.equal(await secondRun, "error");
  assert.deepEqual(errors, [currentError]);
});

test("invalidation prevents a pending request from applying", async () => {
  const controller = createLatestRequestController();
  const pending = deferred<string>();
  const applied: string[] = [];
  const run = runLatestRequest(controller, () => pending.promise, {
    onSuccess: (value) => applied.push(value),
  });

  controller.invalidate();
  pending.resolve("obsolete");
  assert.equal(await run, "stale");
  assert.deepEqual(applied, []);
});

test("stale request does not settle loading owned by the current request", async () => {
  const controller = createLatestRequestController();
  const first = deferred<string>();
  const second = deferred<string>();
  const settled: string[] = [];

  const firstRun = runLatestRequest(controller, () => first.promise, {
    onSuccess: () => undefined,
    onSettled: () => settled.push("first"),
  });
  const secondRun = runLatestRequest(controller, () => second.promise, {
    onSuccess: () => undefined,
    onSettled: () => settled.push("second"),
  });

  first.resolve("old");
  assert.equal(await firstRun, "stale");
  assert.deepEqual(settled, []);

  second.resolve("new");
  assert.equal(await secondRun, "success");
  assert.deepEqual(settled, ["second"]);
});
