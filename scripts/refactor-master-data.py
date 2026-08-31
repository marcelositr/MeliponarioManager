from pathlib import Path

source = Path("src-tauri/src/master_data.rs")
text = source.read_text()

melip_start = text.index("pub async fn edit_meliponary(")
species_start = text.index("pub async fn edit_species(")
box_start = text.index("pub async fn edit_box(")
colony_start = text.index("pub async fn edit_colony(")
tests_start = text.index("#[cfg(test)]\nmod tests {")

header = text[:melip_start].rstrip()
meliponaries = text[melip_start:species_start].rstrip()
species = text[species_start:box_start].rstrip()
boxes = text[box_start:colony_start].rstrip()
colonies = text[colony_start:tests_start].rstrip()

tests_block = text[tests_start:]
brace = tests_block.index("{")
tests_body = tests_block[brace + 1:].rstrip()
if not tests_body.endswith("}"):
    raise SystemExit("master_data test module closing brace not found")
tests_body = tests_body[:-1].strip("\n")

module_dir = Path("src-tauri/src/master_data")
module_dir.mkdir(exist_ok=True)
(module_dir / "meliponaries.rs").write_text("use super::*;\n\n" + meliponaries + "\n")
(module_dir / "species.rs").write_text("use super::*;\n\n" + species + "\n")
(module_dir / "boxes.rs").write_text("use super::*;\n\n" + boxes + "\n")
(module_dir / "colonies.rs").write_text("use super::*;\n\n" + colonies + "\n")
(module_dir / "tests.rs").write_text(tests_body + "\n")

facade = header + """

mod boxes;
mod colonies;
mod meliponaries;
mod species;

pub use boxes::{delete_box, edit_box};
pub use colonies::{delete_colony, edit_colony};
pub use meliponaries::{archive_meliponary, delete_meliponary, edit_meliponary, reactivate_meliponary};
pub use species::{archive_species, delete_species, edit_species, reactivate_species};

#[cfg(test)]
mod tests;
"""
source.write_text(facade)

required_exports = [
    "edit_meliponary", "archive_meliponary", "reactivate_meliponary", "delete_meliponary",
    "edit_species", "archive_species", "reactivate_species", "delete_species",
    "edit_box", "delete_box", "edit_colony", "delete_colony",
]
combined = source.read_text() + "".join(p.read_text() for p in module_dir.glob("*.rs"))
missing = [name for name in required_exports if name not in combined]
if missing:
    raise SystemExit(f"missing master-data entrypoints after split: {missing}")
