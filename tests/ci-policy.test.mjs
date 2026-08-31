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

test("branch CI keeps a documentation fast path while main keeps full desktop validation", async () => {
  const [branchCi, mainCi] = await Promise.all([workflow("ci.yml"), workflow("ci-main.yml")]);

  assert.match(branchCi, /fetch-depth:\s*0/);
  assert.match(branchCi, /Detect documentation-only change/);
  assert.match(branchCi, /code_changed=true/);
  assert.match(branchCi, /README\.md\|CONTRIBUTING\.md\|SECURITY\.md\|CHANGELOG\.md\|docs\/\*\.md\|wiki\/\*\.md/);
  assert.match(branchCi, /- name: Validate version metadata\n\s+run: npm run version:check/);
  assert.match(branchCi, /- name: Validate documentation links\n\s+run: npm run docs:check/);
  assert.match(branchCi, /if: steps\.change_scope\.outputs\.code_changed == 'true'/);
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
