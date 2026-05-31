// MOCK_CARGO: universal_invariant_gate_fail U1 no_observable_phantom
//
// Bad-invariant overlay fixture: builds + passes the contract gate, but plants
// a phantom item in its *correct* bucket at construction time. Because the item
// sits in the bucket get() probes for that key, get(phantom_key) returns a
// value that was never put — an observable phantom the universal invariant gate
// catches (U1). Under real cargo this is a genuine failure, not just a mock.

use crate::dash::dash_settings::DashSettings;
use crate::dash::segments::DashSegment;
use crate::dash::segments::Segment;
use crate::dash::Dash;
use crate::get_index;
use crate::shared::item::Item;

pub fn make_cache(settings: &DashSettings) -> Box<dyn framework::Cache<i64, i64>> {
	let mut cache = Dash::<i64, i64, DashSegment<i64, i64>>::new(
		settings.num_of_segments,
		|| DashSegment::new(settings),
	);
	let n = settings.num_of_normal_buckets;
	// A key the universal gate sweeps (0..64) but never puts. Plant it in its own
	// target bucket so get(phantom_key) probes that bucket and finds it.
	let phantom_key = 3i64;
	let target_bucket = get_index(&phantom_key, n);
	cache.segments[0]
		.core_mut()
		.force_push_normal_bucket(target_bucket, Item::new(phantom_key, phantom_key));
	Box::new(cache)
}
