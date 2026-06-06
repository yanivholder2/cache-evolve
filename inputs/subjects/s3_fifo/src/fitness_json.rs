//! Plugin JSON contract writer for `src/bin/evolve_*.rs`.
//!
//! Duplicated from `evolve/bench/src/fitness_json.rs` — the bench crate is std-only
//! and can't depend on this crate, so the helper lives in both places.
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
		"{{\"dim\":\"{dim}\",\"values\":{values_json},\"summary\":{summary},\"higher_is_better\":{higher_is_better}}}",
	)
}
