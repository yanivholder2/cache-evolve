//! Plugin JSON contract writer (std-only).
//!
//! Mirrors `src/fitness_json.rs` in the main crate. Duplicated because the bench
//! crate is std-only by design (no serde, no workspace deps), and the helper is
//! small enough that duplication beats a shared crate.
//!
//! Contract: `{ "dim": <str>, "values": <json-array>, "summary": <number>, "higher_is_better": <bool> }`.

use std::fmt::Display;

/// Format a plugin JSON record. `values_json` must already be a well-formed JSON array literal.
pub fn write_plugin_json(dim: &str, values_json: &str, summary: impl Display, higher_is_better: bool) -> String {
	assert!(
		dim.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
		"dim must match [a-z0-9-]+, got {dim:?}",
	);
	format!(
		"{{\n  \"dim\": \"{dim}\",\n  \"values\": {values_json},\n  \"summary\": {summary},\n  \"higher_is_better\": {higher_is_better}\n}}\n",
	)
}
