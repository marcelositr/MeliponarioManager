import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

const filesPanel = fs.readFileSync(new URL("../src/components/MeliponaryFilesPanel.tsx", import.meta.url), "utf8");
const recordCenter = fs.readFileSync(new URL("../src/components/OperationalRecordCenter.tsx", import.meta.url), "utf8");
const assetsPage = fs.readFileSync(new URL("../src/pages/AssetsPage.tsx", import.meta.url), "utf8");
const assetsModule = [
  assetsPage,
  fs.readFileSync(new URL("../src/pages/assets/AssetsMaintenancePanel.tsx", import.meta.url), "utf8"),
  fs.readFileSync(new URL("../src/pages/assets/AssetsPhotoLibrary.tsx", import.meta.url), "utf8"),
  fs.readFileSync(new URL("../src/pages/assets/presentation.ts", import.meta.url), "utf8"),
].join("\n");
const dataPage = fs.readFileSync(new URL("../src/pages/DataManagementPage.tsx", import.meta.url), "utf8");
const sidebar = fs.readFileSync(new URL("../src/components/Sidebar.tsx", import.meta.url), "utf8");
const preview = fs.readFileSync(new URL("../src/components/InspectionPhotoPreview.tsx", import.meta.url), "utf8");
const previewBackend = fs.readFileSync(new URL("../src-tauri/src/photo_preview.rs", import.meta.url), "utf8");
const tauriLib = fs.readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
const dataManagementSources = [
  "../src-tauri/src/data_management.rs",
  "../src-tauri/src/data_management/backup.rs",
  "../src-tauri/src/data_management/restore.rs",
  "../src-tauri/src/data_management/exports.rs",
].map((path) => fs.readFileSync(new URL(path, import.meta.url), "utf8"));
const dataManagement = dataManagementSources.join("\n");
const styles = fs.readFileSync(new URL("../src/styles/files.css", import.meta.url), "utf8");

test("managed files stay contextual to the meliponary record center", () => {
  assert.match(recordCenter, /MeliponaryFilesPanel/);
  assert.match(recordCenter, /meliponaryId=\{meliponaryId\}/);
  assert.doesNotMatch(sidebar, /view:\s*["']files["']/);
});

test("attachments use native selection and expose safe file actions", () => {
  assert.match(filesPanel, /@tauri-apps\/plugin-dialog/);
  assert.match(filesPanel, /Anexar arquivo…/);
  assert.match(filesPanel, /openManagedAttachment/);
  assert.match(filesPanel, /revealManagedAttachment/);
  assert.match(filesPanel, /Arquivo não encontrado/);
  assert.doesNotMatch(filesPanel, /Caminho local/);
});

test("inspection photos use native picker, human context and lazy previews", () => {
  assert.match(assetsModule, /Selecionar foto/);
  assert.match(assetsModule, /openInspectionPhoto/);
  assert.match(assetsModule, /revealInspectionPhoto/);
  assert.match(assetsModule, /InspectionPhotoPreview/);
  assert.doesNotMatch(assetsModule, /inspectionId\.slice/);
  assert.doesNotMatch(assetsModule, /Caminho local da foto/);
  assert.match(preview, /IntersectionObserver/);
  assert.match(preview, /loading="lazy"/);
  assert.match(previewBackend, /MAX_PREVIEW_BYTES:\s*u64\s*=\s*384 \* 1024/);
});

test("data management exposes managed-file diagnostics", () => {
  assert.match(dataPage, /Diagnóstico de arquivos/);
  assert.match(dataPage, /diagnoseManagedFiles/);
  assert.match(dataPage, /Arquivos ausentes/);
  assert.match(dataPage, /Arquivos sem registro/);
  assert.match(dataPage, /Nenhuma inconsistência de arquivos foi encontrada/);
});

test("backup and JSON semantics are explicit and versioned across the data-management module", () => {
  assert.match(dataManagement, /BACKUP_FORMAT_VERSION:\s*u32\s*=\s*1/);
  assert.match(dataManagement, /PORTABLE_FORMAT_VERSION:\s*u32\s*=\s*1/);
  assert.match(dataManagement, /assets_embedded:\s*false/);
  assert.match(dataManagement, /managed_attachments/);
  assert.match(dataManagement, /inspection_photos/);
  assert.match(dataManagement, /PRAGMA integrity_check/);
  assert.doesNotMatch(dataManagement, /import_portable_json/);
});

test("window state and compact files styling are wired into the desktop", () => {
  assert.match(tauriLib, /tauri_plugin_window_state::Builder::default\(\)\.build\(\)/);
  assert.match(styles, /\.managed-files-table/);
  assert.match(styles, /\.photo-thumbnail/);
  assert.match(styles, /@media \(max-width: 899px\)/);
});
