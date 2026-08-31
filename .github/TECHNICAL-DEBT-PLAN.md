# Technical debt execution plan

> Temporary execution checkpoint for `refactor/technical-debt`.
> Remove this file before final integration.

## Goal

Resolve the post-audit technical debt in a single branch without changing product behavior, migrations, persisted data contracts or public user flows unless a defect is discovered during validation.

## Working rules

- one branch only: `refactor/technical-debt`;
- refactor by natural responsibility boundaries, not arbitrary line counts;
- preserve IPC command names and database behavior;
- no existing migration may be edited;
- add or adapt regression tests whenever a moved boundary needs protection;
- keep fast PR CI green throughout the work;
- manual application test only after all technical blocks are finished;
- final integration remains squash-only.

## 1. Backend decomposition

- [x] Decompose `record_corrections.rs` by correction domain/responsibility.
  - facade/shared audit helpers remain in `record_corrections.rs`;
  - operational facts moved to `record_corrections/facts.rs`;
  - movement/document corrections moved to `record_corrections/movement_documents.rs`;
  - division/occupancy corrections moved to `record_corrections/history.rs`;
  - public function names and existing test module preserved.
- [x] Decompose `agenda.rs` beyond the already-separated test module where natural boundaries exist.
  - public task types and shared audit helpers remain in `agenda.rs`;
  - queries and summary projections moved to `agenda/queries.rs`;
  - manual task lifecycle moved to `agenda/manual.rs`;
  - fact-derived reconciliation moved to `agenda/derived.rs`;
  - existing `agenda/tests.rs` and public function names preserved.
- [x] Decompose `data_management.rs` into backup/restore/export/diagnostics responsibilities where safe.
  - shared filesystem/checksum/schema helpers remain in `data_management.rs`;
  - backup creation moved to `data_management/backup.rs`;
  - restore validation, staging, rollback and startup application moved to `data_management/restore.rs`;
  - portable JSON and management-report export moved to `data_management/exports.rs`;
  - managed-file diagnostics were already correctly isolated in `managed_files.rs` and were not duplicated.
- [x] Decompose `master_data.rs` by master-data domain while preserving command/service contracts.
  - meliponary, species, box and colony operations now live in entity-specific submodules;
  - shared validation/read helpers remain in the facade;
  - inline tests moved to `master_data/tests.rs`.
- [x] Decompose `movements.rs` by movement type/lifecycle where safe.
  - transactional creation and type dispatch remain cohesive in `movements/creation.rs` because internal/external transfer share one transaction boundary;
  - read projections moved to `movements/queries.rs`;
  - inline tests moved to `movements/tests.rs`;
  - transport return lifecycle remains separately owned by `transport.rs`.
- [x] Decompose `repository.rs` into repository primitives/helpers without leaking SQLx details across IPC.
  - core entity CRUD/queries moved to `repository/entities.rs`;
  - box occupancy mutation moved to `repository/occupancy.rs`;
  - `AppError` and shared input validation remain in the facade;
  - inline tests moved to `repository/tests.rs`.
- [x] Decompose `transport.rs` by lifecycle/queries where safe.
  - completion/reopen lifecycle moved to `transport/lifecycle.rs`;
  - return queries moved to `transport/queries.rs`;
  - Tauri wrappers moved to `transport/commands.rs`;
  - active-return helper remains shared in the facade and tests moved to `transport/tests.rs`.
- [x] Decompose `reversals.rs` by reversal domain where safe.
  - lifecycle reversal moved to `reversals/lifecycle.rs`;
  - movement reversal moved to `reversals/movements.rs`;
  - historical guards/box restoration remain shared in the facade;
  - inline tests moved to `reversals/tests.rs`.
- [x] Re-run orphan-module and active-module audit after decomposition.
  - no one-shot refactor workflow or `scripts/refactor-*` file remains;
  - previously removed `stage3_migration_tests` and `record_view*` modules remain absent;
  - all new submodules are reachable from active parent modules.

## 2. Frontend decomposition

- [x] Decompose `MovementsPage.tsx` into cohesive transport/document/action UI units while avoiding prop-drilling explosion.
  - orchestration and transport mutations remain in the page facade;
  - history, creation, document workflows and pure presentation helpers live under `src/pages/movements/`;
  - transport hardening tests now validate the module as one architectural unit.
- [x] Decompose `AgendaPage.tsx` into cohesive task/list/dialog units while preserving shared reload/mutation behavior.
  - contextual queries and all operational mutations remain in the facade;
  - list/summary, creation, task dialogs, forms and presentation helpers live under `src/pages/agenda/`;
  - Stage 4 API-contract assertions remain intact.
- [x] Decompose `AssetsPage.tsx` into cohesive photo/maintenance units while preserving selection and feedback behavior.
  - native file selection and open/reveal effects remain in the facade;
  - maintenance history, photo library and presentation helpers live under `src/pages/assets/`;
  - photo/file tests validate the full module instead of a monolithic source file.
- [x] Review other large pages after these changes; no additional split is justified inside this debt scope without mixing in unrelated product work.

## 3. Desktop runtime coverage

- [x] Expand desktop smoke coverage beyond startup with a non-destructive runtime interaction.
  - main validation installs the native WebKit WebDriver and pinned `tauri-driver 2.0.6`;
  - `scripts/desktop-webdriver-smoke.py` launches the real Tauri binary through W3C WebDriver;
  - the smoke asserts the initial `Visão geral` heading, clicks `Abrir Agenda`, then asserts the `Agenda` heading.
- [x] Assess critical native-dialog/capability coverage.
  - dialog permissions remain structurally asserted in the UI hardening suite;
  - OS-native modal automation is intentionally not mixed into the portable desktop smoke because it is outside the WebView contract and substantially more brittle in headless CI.
- [x] Keep destructive restore testing out of the default runtime smoke suite.

## 4. Dependency/security maintenance

- [x] Keep RustSec warnings visible and document upstream GTK/Tauri ownership without incompatible overrides.
- [x] Review current Tauri/GTK releases.
  - reviewed on 2026-08-31;
  - `tauri 2.11.5` remains current stable and upstream Tauri development still uses GTK `0.18` on Linux;
  - the GTK4/WebKitGTK 6 migration is still upstream work, so no safe dependency override is applied.
- [x] Optimize `cargo-audit` execution time without weakening the independent security gate.
  - pinned `cargo-audit 0.22.2 --locked` remains unchanged;
  - `Swatinem/rust-cache` now preserves the Cargo tool binary/install metadata with target caching disabled for this job.
- [x] Preserve npm high/critical audit coverage.

## 5. Repository/CI maintenance

- [ ] Re-check branch/PR CI timing after refactors and cache changes.
- [x] Re-check pinned GitHub Actions and checkout credential policy through `tests/ci-policy.test.mjs`.
- [x] Confirm documentation link checker still covers all maintained docs/wiki surfaces.

## 6. Final automated validation

- [ ] Review complete diff against `main`.
- [ ] Confirm migrations remain unchanged.
- [ ] Confirm no secrets, personal paths, debug leftovers or temporary workflows remain.
- [ ] Confirm fast CI green on final branch HEAD.
- [ ] Confirm real desktop WebDriver validation green on the final technical state.
- [ ] Confirm dependency security audit green on final branch HEAD/PR.
- [ ] Remove this temporary plan file and any temporary validation tooling.
- [ ] Open one final PR against `main`.

## 7. Final manual gate

Only after sections 1–6 are complete:

- [ ] Launch application with existing data.
- [ ] Exercise major navigation and refactored pages.
- [ ] Verify CSV save dialog/export.
- [ ] Verify automatic full backup and reported path.
- [ ] Verify managed file open/reveal when test data is available.
- [ ] Avoid destructive restore against real data.

## 8. Integration

- [ ] Mark final PR ready only after manual gate passes.
- [ ] Squash merge into `main`.
- [ ] Confirm `Main validation` green on integrated SHA.
- [ ] Confirm security audit state on integrated SHA.
