import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const workflows = ["ci.yml", "ci-main.yml", "build-bundles.yml", "wiki.yml"];

async function workflow(name) {
  return readFile(new URL(`../.github/workflows/${name}`, import.meta.url), "utf8");
}

test("external GitHub Actions are pinned to immutable full SHAs", async () => {
  for (const name of workflows) {
    const source = await workflow(name);
    const uses = [...source.matchAll(/^\s*uses:\s*([^\s#]+)(?:\s*#.*)?$/gm)].map((match) => match[1]);
    assert.ok(uses.length > 0, `${name} should use at least one external action`);
    for (const reference of uses) {
      assert.match(reference, /^[^@\s]+@[0-9a-f]{40}$/, `${name}: ${reference} must be pinned by full SHA`);
    }
  }
});

test("branch CI remains fast while main keeps full desktop validation", async () => {
  const [branchCi, mainCi] = await Promise.all([workflow("ci.yml"), workflow("ci-main.yml")]);

  assert.match(branchCi, /npm run docs:check/);
  assert.match(branchCi, /cargo check --locked/);
  assert.match(branchCi, /cargo clippy --locked --all-targets -- -D warnings/);
  assert.match(branchCi, /cargo test --locked/);
  assert.doesNotMatch(branchCi, /tauri -- build --no-bundle/);

  assert.match(mainCi, /npm run docs:check/);
  assert.match(mainCi, /tauri -- build --no-bundle/);
  assert.match(mainCi, /Smoke test desktop startup/);
  assert.match(mainCi, /timeout 8s xvfb-run -a src-tauri\/target\/release\/meliponariomanager/);
});

test("Rust build cache is enabled where compilation is expensive", async () => {
  for (const name of ["ci.yml", "ci-main.yml", "build-bundles.yml"]) {
    const source = await workflow(name);
    assert.match(source, /Swatinem\/rust-cache@[0-9a-f]{40}/, `${name} should cache Rust builds`);
    assert.match(source, /workspaces:\s*src-tauri -> target/);
  }
});
