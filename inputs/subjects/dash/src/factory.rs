//! Contract-protected cache constructor.
//!
//! The baseline returns the default Dash topology with `DashSegment`. This file
//! is the protected glue between the JNI surface and the extension points:
//! variants cannot overlay it, but it dispatches through the overlayable
//! `geometry` seam so a variant can reshape the topology by overlaying
//! `src/shared/geometry.rs` alone.
//!
//! Both the eval path (`make_cache`) and the invariant gate's
//! `P6 capacity_conservation` resolve geometry through `resolve_topology`, so the
//! gate exercises the exact code the eval path runs — not a parallel copy.

use crate::dash::dash_settings::DashSettings;
use crate::dash::segments::DashSegment;
use crate::dash::Dash;
use crate::shared::geometry::BaselineGeometry;
use crate::shared::traits::geometry::BucketGeometry;

/// Resolve the topology through the overlayable `geometry` seam and enforce the
/// capacity-conservation contract: the seam may reshape the topology but must
/// preserve the total capacity budget and yield a structurally valid layout.
/// Returns the resolved settings, or an error naming the violation.
pub fn resolve_topology(settings: &DashSettings) -> Result<DashSettings, String> {
	let budget = settings.capacity();
	let resolved = settings.with_topology(BaselineGeometry::for_settings(settings).layout(budget));
	check_conservation(budget, &resolved)?;
	Ok(resolved)
}

/// The capacity-conservation predicate the gate and the eval path share: a
/// resolved topology must be structurally valid and hold exactly `budget` slots.
pub fn check_conservation(budget: usize, resolved: &DashSettings) -> Result<(), String> {
	if let Err(problems) = resolved.validate() {
		return Err(format!("geometry produced an invalid topology: {}", problems.join(", ")));
	}
	if resolved.capacity() != budget {
		return Err(format!("budget {budget} but geometry yields {}", resolved.capacity()));
	}
	Ok(())
}

pub fn make_cache(settings: &DashSettings) -> Box<dyn framework::Cache<i64, i64>> {
	// The harness fixes the total capacity; the geometry seam only chooses how to
	// factor that budget into segments, buckets, and slots. A variant whose
	// geometry violates conservation is rejected by the invariant gate before it
	// ever reaches eval; the expect here names the seam if one somehow slips past.
	let resolved = resolve_topology(settings)
		.expect("geometry violated capacity conservation; the invariant gate should have rejected this variant");
	let cache = Dash::<i64, i64, DashSegment<i64, i64>>::new(
		resolved.num_of_segments,
		|| DashSegment::new(&resolved),
	);
	Box::new(cache)
}
