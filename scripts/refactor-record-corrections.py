from pathlib import Path

source = Path("src-tauri/src/record_corrections.rs")
text = source.read_text()

facts_start = text.index("pub async fn correct_inspection(")
movement_start = text.index("pub async fn correct_movement_details(")
history_start = text.index("pub async fn correct_division(")
void_start = text.index("async fn void_fact(")
tests_start = text.index("#[cfg(test)]\nmod tests;")

header = text[:facts_start].rstrip()
facts = text[facts_start:movement_start].rstrip()
movement = text[movement_start:history_start].rstrip()
history = text[history_start:void_start].rstrip()
shared_void = text[void_start:tests_start].rstrip()

module_dir = Path("src-tauri/src/record_corrections")
module_dir.mkdir(exist_ok=True)
(module_dir / "facts.rs").write_text("use super::*;\n\n" + facts + "\n")
(module_dir / "movement_documents.rs").write_text("use super::*;\n\n" + movement + "\n")
(module_dir / "history.rs").write_text("use super::*;\n\n" + history + "\n")

facade = header + """

mod facts;
mod history;
mod movement_documents;

pub use facts::{
    correct_event, correct_feeding, correct_inspection, correct_maintenance, correct_production,
    void_event, void_feeding, void_inspection, void_maintenance, void_production,
};
pub use history::{correct_division, correct_occupancy, void_division};
pub use movement_documents::{
    correct_movement_details, update_movement_document, void_movement_document, void_transport,
};

""" + shared_void + "\n\n#[cfg(test)]\nmod tests;\n"
source.write_text(facade)

required_exports = [
    "correct_inspection", "void_inspection", "correct_feeding", "void_feeding",
    "correct_production", "void_production", "correct_maintenance", "void_maintenance",
    "correct_event", "void_event", "correct_movement_details", "void_transport",
    "update_movement_document", "void_movement_document", "correct_division",
    "void_division", "correct_occupancy",
]
combined = source.read_text() + "".join(p.read_text() for p in module_dir.glob("*.rs"))
missing = [name for name in required_exports if name not in combined]
if missing:
    raise SystemExit(f"missing entrypoints after split: {missing}")
