from pathlib import Path

source = Path("src-tauri/src/transport.rs")
text = source.read_text()

lifecycle_start = text.index("pub async fn has_open_transport_for_colony(")
queries_start = text.index("pub async fn list_by_colony(")
active_return_start = text.index("async fn get_active_return(")
commands_start = text.index("#[tauri::command]\npub async fn complete_transport(")
tests_start = text.index("#[cfg(test)]\nmod tests {")

header = text[:lifecycle_start].rstrip()
lifecycle = text[lifecycle_start:queries_start].rstrip()
queries = text[queries_start:active_return_start].rstrip()
active_return = text[active_return_start:commands_start].rstrip()
commands = text[commands_start:tests_start].rstrip()

tests_block = text[tests_start:]
brace = tests_block.index("{")
tests_body = tests_block[brace + 1:].rstrip()
if not tests_body.endswith("}"):
    raise SystemExit("transport test module closing brace not found")
tests_body = tests_body[:-1].strip("\n")

module_dir = Path("src-tauri/src/transport")
module_dir.mkdir(exist_ok=True)
(module_dir / "lifecycle.rs").write_text("use super::*;\n\n" + lifecycle + "\n")
(module_dir / "queries.rs").write_text("use super::*;\n\n" + queries + "\n")
(module_dir / "commands.rs").write_text("use super::*;\n\n" + commands + "\n")
(module_dir / "tests.rs").write_text(tests_body + "\n")

facade = header + "\n\n" + active_return + """

mod commands;
mod lifecycle;
mod queries;

pub use commands::{complete_transport, list_transport_returns, reopen_transport};
pub use lifecycle::{complete, has_open_transport_for_colony, reopen};
pub use queries::list_by_colony;

#[cfg(test)]
mod tests;
"""
source.write_text(facade)

required_exports = [
    "complete", "reopen", "has_open_transport_for_colony", "list_by_colony",
    "complete_transport", "list_transport_returns", "reopen_transport",
]
combined = source.read_text() + "".join(p.read_text() for p in module_dir.glob("*.rs"))
missing = [name for name in required_exports if name not in combined]
if missing:
    raise SystemExit(f"missing transport entrypoints after split: {missing}")
