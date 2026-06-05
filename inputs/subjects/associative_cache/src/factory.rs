//! Contract-protected cache constructor.
//!
//! The baseline returns the default `AssociativeCache` topology. This file is the
//! protected glue between the JNI surface and the extension points: variants
//! cannot overlay it, but it dispatches through the overlayable `geometry` seam
//! so a variant can reshape the bucket layout by overlaying
//! `src/shared/geometry.rs` alone.
//!
//! Both the eval path (`make_cache`) and the invariant gate's
//! `P4 capacity_conservation` resolve geometry through `resolve_geometry`, so the
//! gate exercises the exact code the eval path runs — not a parallel copy.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::associative_cache::AssociativeCache;
use crate::shared::geometry::BaselineGeometry;
use crate::shared::traits::geometry::BucketGeometry;

/// Resolve the bucket layout through the overlayable `geometry` seam and enforce
/// the capacity-conservation contract: the seam may reshape the layout but must
/// preserve the total capacity budget and yield a structurally valid split.
/// Returns the resolved settings, or an error naming the violation.
pub fn resolve_geometry(settings: &AssociativeCacheSettings) -> Result<AssociativeCacheSettings, String> {
	let budget = settings.capacity();
	let (num_of_buckets, bucket_size) = BaselineGeometry::for_settings(settings).layout(budget);
	let resolved = settings.with_geometry(num_of_buckets, bucket_size);
	check_conservation(budget, &resolved)?;
	Ok(resolved)
}

/// The capacity-conservation predicate the gate and the eval path share: a
/// resolved layout must be structurally valid and hold exactly `budget` slots.
pub fn check_conservation(budget: usize, resolved: &AssociativeCacheSettings) -> Result<(), String> {
	if let Err(problems) = resolved.validate() {
		return Err(format!("geometry produced an invalid layout: {}", problems.join(", ")));
	}
	if resolved.capacity() != budget {
		return Err(format!("budget {budget} but geometry yields {}", resolved.capacity()));
	}
	Ok(())
}

pub fn make_cache(settings: &AssociativeCacheSettings) -> Box<dyn framework::Cache<i64, i64>> {
	// The harness fixes the total capacity; the geometry seam only chooses how to
	// factor that budget into buckets and slots. A variant whose geometry violates
	// conservation is rejected by the invariant gate before it ever reaches eval;
	// the expect here names the seam if one somehow slips past.
	let resolved = resolve_geometry(settings)
		.expect("geometry violated capacity conservation; the invariant gate should have rejected this variant");
	let cache = AssociativeCache::<i64, i64>::new(resolved);
	Box::new(cache)
}
