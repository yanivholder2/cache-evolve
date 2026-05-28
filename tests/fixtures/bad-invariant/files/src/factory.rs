// MOCK_CARGO: invariant_gate_fail P3 slot_legality
//
// Bad-invariant overlay fixture: builds + passes the contract gate (the
// Cache trait still round-trips correctly under normal use) but plants a
// phantom item in the wrong bucket at construction time, violating
// P3 slot_legality.

use crate::dash::dash_settings::DashSettings;
use crate::dash::segments::PromotionColdSegment;
use crate::dash::Dash;
use crate::get_index;
use crate::shared::item::Item;

pub fn make_cache(settings: &DashSettings) -> Box<dyn framework::Cache<i64, i64>> {
	let mut cache = Dash::<i64, i64, PromotionColdSegment<i64, i64>>::new(
		settings.num_of_segments,
		|| PromotionColdSegment::new(settings),
	);
	let n = settings.num_of_normal_buckets;
	// Pick a key whose target bucket is neither 0 nor n-1: that excludes 0
	// from both target and probing ((target+1) % n), guaranteeing slot
	// illegality in bucket 0 independent of the concrete hash.
	let phantom_key = (0i64..(4 * n as i64))
		.find(|k| {
			let t = get_index(k, n);
			t != 0 && t != n - 1
		})
		.expect("no non-edge bucket found in search range");
	cache.segments[0]
		.core_mut()
		.force_push_normal_bucket(0, Item::new(phantom_key, phantom_key));
	Box::new(cache)
}
