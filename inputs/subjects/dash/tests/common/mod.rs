use dash::get_index;

/// Find `count` keys that hash to `target_idx` given `num_slots` total slots.
/// Searches `0..1000` and returns as many as found (may be fewer than `count`).
#[allow(dead_code)]
pub fn find_keys_for_index(target_idx: usize, num_slots: usize, count: usize) -> Vec<i64> {
	(0..1000)
		.filter(|k| get_index(k, num_slots) == target_idx)
		.take(count)
		.collect()
}
