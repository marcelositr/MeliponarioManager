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
- [ ] Decompose `master_data.rs` by master-data domain while preserving command/service contracts.
- [ ] Decompose `movements.rs` by movement type/lifecycle where safe.
- [ ] Decompose `repository.rs` into repository primitives/helpers without leaking SQLx details across IPC.
- [ ] Decompose `transport.rs` by lifecycle/queries where safe.
- [ ] Decompose `reversals.rs` by reversal domain where safe.
- [ ] Re-run orphan-module and active-module audit after decomposition.

## 2. Frontend decomposition

- [ ] Decompose `MovementsPage.tsx` into cohesive transport/document/action UI units while avoiding prop-drilling explosion.
- [ ] Decompose `AgendaPage.tsx` into cohesive task/list/dialog units while preserving shared reload/mutation behavior.
- [ ] Decompose `AssetsPage.tsx` into cohesive photo/maintenance units while preserving selection and feedback behavior.
- [ ] Review other large pages after these changes; split only where a natural boundary is demonstrated.

## 3. Desktop runtime coverage

- [ ] Expand desktop smoke coverage beyond startup with at least one non-destructive runtime interaction.
- [ ] Add coverage for a critical native-dialog/capability path where practical in CI.
- [ ] Keep destructive restore testing out of the default runtime smoke suite.

## 4. Dependency/security maintenance

- [ ] Keep RustSec warnings visible and document upstream GTK/Tauri ownership without incompatible overrides.
- [ ] Review whether current Tauri/GTK releases remove any existing warnings; update only if compatible and justified.
- [ ] Optimize `cargo-audit` execution time without weakening the independent security gate.
- [ ] Preserve npm high/critical audit coverage.

## 5. Repository/CI maintenance

- [ ] Re-check branch/PR CI timing after refactors and cache changes.
- [ ] Re-check pinned GitHub Actions and checkout credential policy.
- [ ] Confirm documentation link checker still covers all maintained docs/wiki surfaces.

## 6. Final automated validation

- [ ] Review complete diff against `main`.
- [ ] Confirm migrations remain unchanged.
- [ ] Confirm no secrets, personal paths, debug leftovers or temporary workflows remain.
- [ ] Confirm fast CI green on final branch HEAD.
- [ ] Confirm dependency security audit green on final branch HEAD.
- [ ] Remove this temporary plan file and any temporary refactor tooling.
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
