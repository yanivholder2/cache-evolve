use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::associative_cache::AssociativeCache;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::get_index;
use crate::test_utils::baseline_settings;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{TestError, TestRunner};

/// Generates valid `AssociativeCacheSettings` across the layout space, so the
/// capacity-conservation property is exercised at many budgets — not just the
/// baseline. `bucket_size` ranges past the swept `bucketSize` the eval path uses,
/// so a budget-dependent variant geometry is covered.
fn valid_settings() -> impl Strategy<Value = AssociativeCacheSettings> {
	(1usize..32, 1usize..128).prop_map(|(num_of_buckets, bucket_size)| AssociativeCacheSettings {
		num_of_buckets,
		bucket_size,
		eviction_policy: EvictionPolicyKind::ClassicLRU,
	})
}

/// Invariant entry point for the AssociativeCache baseline. Delegates to
/// [`run_invariant_suite`] with the baseline settings.
pub struct AssociativeCacheInvariants;

impl framework::Invariants for AssociativeCacheInvariants {
	fn run_invariant_suite() -> Result<(), String> {
		run_invariant_suite(baseline_settings())
	}
}

/// Run all invariant properties against an `AssociativeCache` built from `settings`.
///
/// Uses proptest internally for randomized coverage. Returns `Ok(())` if all
/// invariants hold, `Err(message)` naming the first violation with proptest's
/// shrunk counterexample.
pub fn run_invariant_suite(settings: AssociativeCacheSettings) -> Result<(), String> {
	// Each property uses a fresh `TestRunner`: a reused runner accumulates
	// `successes` across calls, so later properties would exit early.

	// P1: capacity_bound — no bucket ever holds more than its max_size.
	TestRunner::default()
		.run(&vec(0i64..200, 1..120), |ops| {
			let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(settings);
			for &key in &ops {
				if cache.get_and_update_item(&key).is_none() {
					cache.put(key, key);
				}
				for (idx, bucket) in cache.buckets.iter().enumerate() {
					prop_assert!(
						bucket.size() <= bucket.get_max_size(),
						"capacity_bound violated: bucket {idx} has {} items, max={}",
						bucket.size(),
						bucket.get_max_size()
					);
				}
			}
			Ok(())
		})
		.map_err(|e| format_err("P1 capacity_bound", e))?;

	// P2: slot_legality — every resident item lives in the bucket its key hashes to.
	TestRunner::default()
		.run(&vec(0i64..200, 1..120), |ops| {
			let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(settings);
			let num_buckets = cache.buckets.len();
			for &key in &ops {
				if cache.get_and_update_item(&key).is_none() {
					cache.put(key, key);
				}
			}
			for (idx, bucket) in cache.buckets.iter().enumerate() {
				for item in bucket.get_items() {
					prop_assert!(
						get_index(&item.key, num_buckets) == idx,
						"slot_legality violated: key={} sits in bucket {idx} but hashes to {}",
						item.key,
						get_index(&item.key, num_buckets)
					);
				}
			}
			Ok(())
		})
		.map_err(|e| format_err("P2 slot_legality", e))?;

	// P3: no_phantom_items — a resident key is always retrievable, and a
	// never-put key always misses.
	TestRunner::default()
		.run(&vec(0i64..120, 1..120), |ops| {
			let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(settings);
			for &key in &ops {
				if cache.get_and_update_item(&key).is_none() {
					cache.put(key, key);
				}
			}
			let num_buckets = cache.buckets.len();
			// Every item physically present must be retrievable.
			let resident: Vec<i64> = cache
				.buckets
				.iter()
				.flat_map(|b| b.get_items().iter().map(|i| i.key))
				.collect();
			for key in resident {
				prop_assert!(
					cache.get_and_update_item(&key).is_some(),
					"no_phantom_items violated: key={key} present but get returned None"
				);
			}
			// A key that hashes into range but was never inserted must miss when absent.
			for key in 1000i64..1010 {
				let idx = get_index(&key, num_buckets);
				let present = cache.buckets[idx].get_position(&key).is_some();
				if !present {
					prop_assert!(
						cache.get_and_update_item(&key).is_none(),
						"no_phantom_items violated: absent key={key} returned Some"
					);
				}
			}
			Ok(())
		})
		.map_err(|e| format_err("P3 no_phantom_items", e))?;

	// P4: capacity_conservation — the geometry seam may reshape buckets but must
	// preserve the total capacity budget and stay structurally valid. Guards
	// fitness fairness: a variant cannot win by quietly enlarging the cache. Drives
	// `factory::resolve_geometry` — the exact resolution the eval path runs —
	// across a range of valid settings, so a variant whose geometry depends on the
	// budget is checked at many operating points, not just the baseline.
	TestRunner::default()
		.run(&valid_settings(), |settings| {
			let result = crate::factory::resolve_geometry(&settings);
			prop_assert!(
				result.is_ok(),
				"P4 capacity_conservation violated: {}",
				result.unwrap_err()
			);
			Ok(())
		})
		.map_err(|e| format!("P4 capacity_conservation: {e}"))?;

	Ok(())
}

fn format_err(invariant: &str, e: TestError<Vec<i64>>) -> String {
	format!("{invariant}: {e}")
}
