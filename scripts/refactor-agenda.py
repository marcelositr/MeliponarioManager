from pathlib import Path

source = Path("src-tauri/src/agenda.rs")
text = source.read_text()

derived_types_start = text.index("#[derive(Debug, Clone)]\nstruct DerivedTask")
helpers_start = text.index("fn required(")
manual_context_start = text.index("async fn validate_manual_context(")
queries_start = text.index("async fn get_with_time(")
manual_start = text.index("pub async fn create_manual(")
derived_start = text.index("async fn reconcile_derived(")
tests_start = text.index("#[cfg(test)]\nmod tests;")

public_types = text[:derived_types_start].rstrip()
derived_types = text[derived_types_start:helpers_start].rstrip()
shared_helpers = text[helpers_start:manual_context_start].rstrip()
manual_context = text[manual_context_start:queries_start].rstrip()
queries = text[queries_start:manual_start].rstrip()
manual = text[manual_start:derived_start].rstrip()
derived = text[derived_start:tests_start].rstrip()

module_dir = Path("src-tauri/src/agenda")
module_dir.mkdir(exist_ok=True)
(module_dir / "queries.rs").write_text("use super::*;\n\n" + queries + "\n")
(module_dir / "manual.rs").write_text("use super::*;\n\n" + manual_context + "\n\n" + manual + "\n")
(module_dir / "derived.rs").write_text("use super::*;\n\n" + derived_types + "\n\n" + derived + "\n")

facade = public_types + "\n\n" + shared_helpers + """

mod derived;
mod manual;
mod queries;

pub use derived::{
    mark_completed_by_fact_tx, reconcile_all, reconcile_feeding, reconcile_inspection,
    reconcile_maintenance,
};
pub use manual::{cancel, complete_generic, create_manual, duplicate, reschedule, skip};
pub use queries::{get, list, summary};

#[cfg(test)]
mod tests;
"""
source.write_text(facade)

required_exports = [
    "get", "list", "summary", "create_manual", "reschedule", "cancel", "skip",
    "complete_generic", "duplicate", "reconcile_inspection", "reconcile_feeding",
    "reconcile_maintenance", "reconcile_all", "mark_completed_by_fact_tx",
]
combined = source.read_text() + "".join(p.read_text() for p in module_dir.glob("*.rs"))
missing = [name for name in required_exports if name not in combined]
if missing:
    raise SystemExit(f"missing agenda entrypoints after split: {missing}")
