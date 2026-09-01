import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const workflows = [
  "ci.yml",
  "ci-main.yml",
  "build-bundles.yml",
  "wiki.yml",
  "security-audit.yml",
];

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

test("checkout credentials are not persisted in repository workflows", async () => {
  for (const name of workflows) {
    const source = await workflow(name);
    const checkoutCount = [...source.matchAll(/uses: actions\/checkout@[0-9a-f]{40}/g)].length;
    const hardenedCheckoutCount = [...source.matchAll(/persist-credentials:\s*false/g)].length;

    assert.ok(checkoutCount > 0, `${name} should checkout the repository`);
    assert.equal(
      hardenedCheckoutCount,
      checkoutCount,
      `${name} must disable persisted credentials for every checkout`,
    );
  }
});

test("branch and PR CI retain full validation while main adds desktop validation", async () => {
  const [branchCi, mainCi, desktopSmoke] = await Promise.all([
    workflow("ci.yml"),
    workflow("ci-main.yml"),
    readFile(new URL("../scripts/desktop-webdriver-smoke.py", import.meta.url), "utf8"),
  ]);

  assert.match(branchCi, /jobs:\n\s+check:/);
  assert.match(
    branchCi,
    /group: ci-\$\{\{ github\.event_name \}\}-\$\{\{ github\.head_ref \|\| github\.ref_name \}\}/,
  );
  assert.match(branchCi, /cancel-in-progress: true/);
  assert.match(branchCi, /fetch-depth:\s*0/);
  assert.match(branchCi, /Detect documentation-only change/);
  assert.match(branchCi, /code_changed=true/);
  assert.match(branchCi, /README\.md\|CONTRIBUTING\.md\|SECURITY\.md\|CHANGELOG\.md\|docs\/\*\.md\|wiki\/\*\.md/);
  assert.doesNotMatch(branchCi, /Select validation profile/);
  assert.doesNotMatch(branchCi, /field-testing light/);
  assert.match(branchCi, /- name: Validate version metadata\n\s+run: npm run version:check/);
  assert.match(branchCi, /- name: Validate documentation links\n\s+run: npm run docs:check/);
  assert.match(branchCi, /- name: Build frontend\n\s+if: steps\.change_scope\.outputs\.code_changed == 'true'/);
  assert.match(branchCi, /- name: Test frontend\n\s+if: steps\.change_scope\.outputs\.code_changed == 'true'/);
  assert.match(branchCi, /- name: Setup Rust\n\s+if: steps\.change_scope\.outputs\.code_changed == 'true'/);
  assert.match(branchCi, /- name: Check Rust formatting\n\s+if: steps\.change_scope\.outputs\.code_changed == 'true'/);
  assert.match(branchCi, /- name: Generate desktop icons\n\s+if: steps\.change_scope\.outputs\.code_changed == 'true'/);
  assert.match(branchCi, /- name: Validate bundle icon configuration\n\s+if: steps\.change_scope\.outputs\.code_changed == 'true'/);
  assert.match(branchCi, /cargo check --locked/);
  assert.match(branchCi, /cargo clippy --locked --all-targets -- -D warnings/);
  assert.match(branchCi, /cargo test --locked/);
  assert.doesNotMatch(branchCi, /tauri -- build --no-bundle/);

  assert.match(mainCi, /npm run docs:check/);
  assert.match(mainCi, /tauri -- build --no-bundle/);
  assert.match(mainCi, /webkit2gtk-driver/);
  assert.match(mainCi, /cargo install tauri-driver --version 2\.0\.6 --locked/);
  assert.match(mainCi, /Exercise desktop WebView with WebDriver/);
  assert.match(mainCi, /xvfb-run -a python3 scripts\/desktop-webdriver-smoke\.py src-tauri\/target\/release\/meliponariomanager/);
  assert.match(desktopSmoke, /browserName.*wry/s);
  assert.match(desktopSmoke, /Visão geral/);
  assert.match(desktopSmoke, /Abrir Agenda/);
  assert.match(desktopSmoke, /def navigate_by_click\(/);
  assert.match(desktopSmoke, /is_transient_driver_transport_error\(error\)/);
  assert.match(desktopSmoke, /http\.client\.RemoteDisconnected/);
  assert.match(desktopSmoke, /wait_for_heading\(session_id, expected_heading, timeout=3\)/);
  assert.match(desktopSmoke, /retrying the click once/);
  assert.match(desktopSmoke, /navigate_by_click\([\s\S]*?Abrir Agenda[\s\S]*?"Agenda",/);
});

test("Rust build cache is enabled where compilation is expensive", async () => {
  for (const name of ["ci.yml", "ci-main.yml", "build-bundles.yml"]) {
    const source = await workflow(name);
    assert.match(source, /Swatinem\/rust-cache@[0-9a-f]{40}/, `${name} should cache Rust builds`);
    assert.match(source, /workspaces:\s*src-tauri -> target/);
  }
});

test("dependency security audit is isolated, pinned, scheduled and caches its pinned Rust tool", async () => {
  const source = await workflow("security-audit.yml");

  assert.match(source, /package\.json/);
  assert.match(source, /package-lock\.json/);
  assert.match(source, /src-tauri\/Cargo\.toml/);
  assert.match(source, /src-tauri\/Cargo\.lock/);
  assert.match(source, /cron: "45 12 \* \* 1"/);
  assert.match(source, /npm audit --audit-level=high/);
  assert.match(source, /Swatinem\/rust-cache@[0-9a-f]{40}/);
  assert.match(source, /cache-targets:\s*false/);
  assert.match(source, /cache-all-crates:\s*true/);
  assert.match(source, /shared-key:\s*cargo-audit-0\.22\.2/);
  assert.match(source, /cargo install cargo-audit --version 0\.22\.2 --locked/);
  assert.match(source, /working-directory: src-tauri\n\s+run: cargo audit/);
  assert.doesNotMatch(source, /required_status_checks|jobs:\n\s+check:/);
});

test("local desktop commands regenerate ignored bundle icons", async () => {
  const packageJson = JSON.parse(await readFile(new URL("../package.json", import.meta.url), "utf8"));

  assert.equal(packageJson.scripts["predesktop:dev"], "npm run icons");
  assert.equal(packageJson.scripts["predesktop:build"], "npm run icons && npm run bundle:check");
  assert.equal(packageJson.scripts["desktop:dev"], "tauri dev");
  assert.equal(packageJson.scripts["desktop:build"], "tauri build");
});
