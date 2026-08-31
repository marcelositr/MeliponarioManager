from pathlib import Path

source = Path("src-tauri/src/data_management.rs")
text = source.read_text()

shared_start = text.index("fn storage_error(")
restore_validate_start = text.index("async fn validate_database(")
backup_helpers_start = text.index("fn write_backup_manifest(")
restore_runtime_start = text.index("fn remove_if_exists(")
backup_command_start = text.index("#[tauri::command]\npub async fn create_full_backup(")
restore_command_start = text.index("#[tauri::command]\npub async fn stage_restore(")
exports_start = text.index("fn blob_hex(")
tests_start = text.index("#[cfg(test)]\nmod tests;")

header = text[:shared_start].rstrip()
shared = text[shared_start:restore_validate_start].rstrip()
restore_validate = text[restore_validate_start:backup_helpers_start].rstrip()
backup_helpers = text[backup_helpers_start:restore_runtime_start].rstrip()
restore_runtime = text[restore_runtime_start:backup_command_start].rstrip()
backup_command = text[backup_command_start:restore_command_start].rstrip()
restore_command = text[restore_command_start:exports_start].rstrip()
exports = text[exports_start:tests_start].rstrip()

module_dir = Path("src-tauri/src/data_management")
module_dir.mkdir(exist_ok=True)
(module_dir / "backup.rs").write_text(
    "use super::*;\n\n" + backup_helpers + "\n\n" + backup_command + "\n"
)
(module_dir / "restore.rs").write_text(
    "use super::*;\n\n" + restore_validate + "\n\n" + restore_runtime + "\n\n" + restore_command + "\n"
)
(module_dir / "exports.rs").write_text("use super::*;\n\n" + exports + "\n")

facade = header + "\n\n" + shared + """

mod backup;
mod exports;
mod restore;

pub use backup::create_full_backup;
pub use exports::{export_portable_json, generate_management_report};
pub use restore::{apply_pending_restore, stage_restore};

#[cfg(test)]
mod tests;
"""
source.write_text(facade)

required_exports = [
    "create_full_backup", "stage_restore", "apply_pending_restore",
    "export_portable_json", "generate_management_report",
]
combined = source.read_text() + "".join(p.read_text() for p in module_dir.glob("*.rs"))
missing = [name for name in required_exports if name not in combined]
if missing:
    raise SystemExit(f"missing data-management entrypoints after split: {missing}")
