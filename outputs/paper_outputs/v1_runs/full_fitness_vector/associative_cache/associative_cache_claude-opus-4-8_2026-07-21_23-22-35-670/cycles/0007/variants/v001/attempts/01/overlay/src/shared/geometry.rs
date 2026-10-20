//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` here does NOT honour the harness split verbatim. Instead it
//! re-factorises the *same* total capacity into fewer, wider buckets (H01: widen
//! the associativity to give the segmented incumbent more slots per set). It
//! keeps the `BaselineGeometry` type name and `for_settings` constructor the
//! construction glue and the invariant gate reference.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Widening factor: how much to grow per-bucket associativity relative to the
/// harness-provided split. `2` doubles `bucket_size` and halves `num_of_buckets`
/// over the same total capacity.
const WIDEN_FACTOR: usize = 2;

/// Geometry that widens the harness split: fewer, larger buckets over the same
/// total capacity.
#[derive(Debug)]
pub struct BaselineGeometry {
	pub num_of_buckets: usize,
	pub bucket_size: usize,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings. The construction glue
	/// and the invariant gate both build the geometry this way, so a variant that
	/// overlays this file inherits the same entry point.
	pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
		BaselineGeometry {
			num_of_buckets: settings.num_of_buckets,
			bucket_size: settings.bucket_size,
		}
	}
}

impl BucketGeometry for BaselineGeometry {
	/// Re-factorise `total_capacity` into fewer, wider buckets.
	///
	/// The new split is derived from `total_capacity` (not the stored fields) so
	/// the product invariant `new_num_of_buckets * new_bucket_size == total_capacity`
	/// holds by construction. We pick the largest widening factor in `1..=WIDEN_FACTOR`
	/// that divides the harness `num_of_buckets`, so halving the bucket count still
	/// tiles the same budget exactly. If nothing divides evenly (or the budget is
	/// degenerate), we fall back to the seeded split rather than change the total.
	fn layout(&self, total_capacity: usize) -> (usize, usize) {
		let base_buckets = self.num_of_buckets;
		let base_size = self.bucket_size;

		// Degenerate/empty budget: return the seeded split verbatim.
		if total_capacity == 0 || base_buckets == 0 || base_size == 0 {
			return (base_buckets, base_size);
		}

		// Largest factor in 1..=WIDEN_FACTOR dividing base_buckets. `1` always
		// divides, so `factor >= 1` and the fallback is the identity split.
		let mut factor = 1;
		let mut f = WIDEN_FACTOR;
		while f >= 1 {
			if base_buckets % f == 0 {
				factor = f;
				break;
			}
			f -= 1;
		}

		let new_buckets = base_buckets / factor;
		// new_size = total_capacity / new_buckets == base_size * factor exactly,
		// because total_capacity == base_buckets * base_size and factor | base_buckets.
		let new_size = total_capacity / new_buckets;

		// Product-conservation guard: never round in a way that changes the total.
		if new_buckets == 0 || new_size == 0 || new_buckets * new_size != total_capacity {
			return (base_buckets, base_size);
		}

		(new_buckets, new_size)
	}
}
