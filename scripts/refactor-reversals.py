from pathlib import Path

source = Path("src-tauri/src/reversals.rs")
text = source.read_text()

lifecycle_start = text.index("pub async fn reverse_lifecycle(")
movement_start = text.index("pub async fn reverse_movement(")
tests_start = text.index("#[cfg(test)]\nmod tests {")

header = text[:lifecycle_start].rstrip()
lifecycle = text[lifecycle_start:movement_start].rstrip()
movement = text[movement_start:tests_start].rstrip()

tests_block = text[tests_start:]
brace = tests_block.index("{")
tests_body = tests_block[brace + 1:].rstrip()
if not tests_body.endswith("}"):
    raise SystemExit("reversals test module closing brace not found")
tests_body = tests_body[:-1].strip("\n")

module_dir = Path("src-tauri/src/reversals")
module_dir.mkdir(exist_ok=True)
(module_dir / "lifecycle.rs").write_text("use super::*;\n\n" + lifecycle + "\n")
(module_dir / "movements.rs").write_text("use super::*;\n\n" + movement + "\n")
(module_dir / "tests.rs").write_text(tests_body + "\n")

facade = header + """

mod lifecycle;
mod movements;

pub use lifecycle::reverse_lifecycle;
pub use movements::reverse_movement;

#[cfg(test)]
mod tests;
"""
source.write_text(facade)

required_exports = ["reverse_lifecycle", "reverse_movement"]
combined = source.read_text() + "".join(p.read_text() for p in module_dir.glob("*.rs"))
missing = [name for name in required_exports if name not in combined]
if missing:
    raise SystemExit(f"missing reversal entrypoints after split: {missing}")
