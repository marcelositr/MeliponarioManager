from pathlib import Path

source = Path("src-tauri/src/repository.rs")
text = source.read_text()

entities_start = text.index("pub async fn core_summary(")
occupancy_start = text.index("pub async fn place_colony(")
tests_start = text.index("#[cfg(test)]\nmod tests {")

header = text[:entities_start].rstrip()
entities = text[entities_start:occupancy_start].rstrip()
occupancy = text[occupancy_start:tests_start].rstrip()

tests_block = text[tests_start:]
brace = tests_block.index("{")
tests_body = tests_block[brace + 1:].rstrip()
if not tests_body.endswith("}"):
    raise SystemExit("repository test module closing brace not found")
tests_body = tests_body[:-1].strip("\n")

module_dir = Path("src-tauri/src/repository")
module_dir.mkdir(exist_ok=True)
(module_dir / "entities.rs").write_text("use super::*;\n\n" + entities + "\n")
(module_dir / "occupancy.rs").write_text("use super::*;\n\n" + occupancy + "\n")
(module_dir / "tests.rs").write_text(tests_body + "\n")

facade = header + """

mod entities;
mod occupancy;

pub use entities::{
    core_summary, create_box, create_colony, create_meliponary, create_species, list_boxes,
    list_colonies, list_meliponaries, list_species,
};
pub use occupancy::place_colony;

#[cfg(test)]
mod tests;
"""
source.write_text(facade)

required_exports = [
    "core_summary", "create_meliponary", "list_meliponaries", "create_species", "list_species",
    "create_box", "list_boxes", "create_colony", "list_colonies", "place_colony",
]
combined = source.read_text() + "".join(p.read_text() for p in module_dir.glob("*.rs"))
missing = [name for name in required_exports if name not in combined]
if missing:
    raise SystemExit(f"missing repository entrypoints after split: {missing}")
