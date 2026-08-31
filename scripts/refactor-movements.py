from pathlib import Path

source = Path("src-tauri/src/movements.rs")
text = source.read_text()

creation_start = text.index("async fn get(")
queries_start = text.index("pub async fn list_by_colony(")
tests_start = text.index("#[cfg(test)]\nmod tests {")

header = text[:creation_start].rstrip()
creation = text[creation_start:queries_start].rstrip()
queries = text[queries_start:tests_start].rstrip()

tests_block = text[tests_start:]
brace = tests_block.index("{")
tests_body = tests_block[brace + 1:].rstrip()
if not tests_body.endswith("}"):
    raise SystemExit("movements test module closing brace not found")
tests_body = tests_body[:-1].strip("\n")

module_dir = Path("src-tauri/src/movements")
module_dir.mkdir(exist_ok=True)
(module_dir / "creation.rs").write_text("use super::*;\n\n" + creation + "\n")
(module_dir / "queries.rs").write_text("use super::*;\n\n" + queries + "\n")
(module_dir / "tests.rs").write_text(tests_body + "\n")

facade = header + """

mod creation;
mod queries;

pub use creation::create;
pub use queries::{count, list_by_colony};

#[cfg(test)]
mod tests;
"""
source.write_text(facade)

required_exports = ["create", "list_by_colony", "count"]
combined = source.read_text() + "".join(p.read_text() for p in module_dir.glob("*.rs"))
missing = [name for name in required_exports if name not in combined]
if missing:
    raise SystemExit(f"missing movement entrypoints after split: {missing}")
