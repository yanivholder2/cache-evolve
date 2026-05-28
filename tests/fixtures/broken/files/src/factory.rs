// MOCK_CARGO: build_gate_fail
//
// Broken-overlay fixture: overlays the subject's `src/factory.rs` with code
// that references a non-existent symbol so the build gate fails.

use crate::dash::dash_settings::DashSettings;

pub fn make_cache(_settings: &DashSettings) -> Box<dyn framework::Cache<i64, i64>> {
	// build-gate failure: `nonexistent_helper` is not declared anywhere.
	crate::dash::nonexistent_helper()
}
