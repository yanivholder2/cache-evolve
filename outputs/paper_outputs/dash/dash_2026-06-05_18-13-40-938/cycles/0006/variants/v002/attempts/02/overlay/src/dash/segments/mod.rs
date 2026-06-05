mod dash_segment;
mod segment;

pub use dash_segment::DashSegment;
pub use segment::Segment;

use super::dash_bucket::new_bucket;
use super::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// Put destination for an item, resolved from its key.
#[derive(Debug, Clone, Copy)]
pub enum Slot {
	Stash,
	Target,
	Probing,
}

pub struct SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	pub(crate) buckets: Vec<Box<dyn Bucket<K, V>>>,
	pub(crate) segment_size: usize,
	pub(crate) stash_buckets: Vec<Box<dyn Bucket<K, V>>>,
	pub(crate) stash_size: usize,
	// Skewed bucket distribution: number of target (0..num_target_buckets)
	// and probing (num_target_buckets..segment_size) buckets.
	pub(crate) num_target_buckets: usize,
	pub(crate) num_probing_buckets: usize,
}

// Manual Debug: `Box<dyn Bucket>` is not `Debug` (the `Bucket` trait has no
// Debug supertrait), so render each bucket through its items instead of deriving.
impl<K, V> Debug for SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("SegmentCore")
			.field("buckets", &self.buckets.iter().map(|b| b.get_items()).collect::<Vec<_>>())
			.field("segment_size", &self.segment_size)
			.field("stash_buckets", &self.stash_buckets.iter().map(|b| b.get_items()).collect::<Vec<_>>())
			.field("stash_size", &self.stash_size)
			.finish()
	}
}

impl<K, V> SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	/// Creates a new SegmentCore with fallible bucket initialization.
	/// Returns `None` if the settings are invalid.
	pub fn try_new(settings: &DashSettings) -> Option<Self> {
		if let Err(problems) = settings.validate() {
			log::error!("SegmentCore.try_new: validation failed: {}", problems.join(", "));
			return None;
		}
		info!("SegmentCore.try_new: settings={settings:?}");

		// Compute skewed bucket distribution: target gets ~35.7% (10/28), probing gets ~64.3% (18/28)
		let num_target_buckets = (settings.num_of_normal_buckets * 10) / 28;
		let num_target_buckets = num_target_buckets.max(1); // Ensure at least 1 target bucket
		let num_probing_buckets = settings.num_of_normal_buckets - num_target_buckets;

		let buckets: Vec<_> = (0..settings.num_of_normal_buckets)
			.map(|_| new_bucket(settings.bucket_size, settings.eviction_policy.build()))
			.collect();
		let stash_buckets: Vec<_> = (0..settings.num_of_stash_buckets)
			.map(|_| new_bucket(settings.bucket_size, settings.eviction_policy.build()))
			.collect();
		Some(SegmentCore {
			segment_size: settings.num_of_normal_buckets,
			stash_size: settings.num_of_stash_buckets,
			num_target_buckets,
			num_probing_buckets,
			buckets,
			stash_buckets,
		})
	}

	/// Creates a new SegmentCore.
	/// # Panics
	/// Panics if initialization fails (e.g., invalid settings).
	pub fn new(settings: &DashSettings) -> Self {
		Self::try_new(settings).expect("SegmentCore::new: invalid settings")
	}

	/// Compute the target bucket index for a key, accounting for tier split.
	/// For keys with baseline < num_target_buckets, this is the baseline.
	/// For keys with baseline >= num_target_buckets, this wraps to a target tier bucket.
	pub(crate) fn target_bucket_for_key(&self, key: &K) -> usize {
		let baseline = get_index(key, self.segment_size);
		if baseline < self.num_target_buckets {
			baseline
		} else {
			(baseline - self.num_target_buckets) % self.num_target_buckets
		}
	}

	/// Compute the probing bucket index for a key, accounting for tier split.
	/// For keys with baseline < num_target_buckets, maps to probing tier [num_target_buckets..].
	/// For keys with baseline >= num_target_buckets, this is the baseline.
	pub(crate) fn probing_bucket_for_key(&self, key: &K) -> usize {
		let baseline = get_index(key, self.segment_size);
		if baseline < self.num_target_buckets {
			self.num_target_buckets + (baseline % self.num_probing_buckets)
		} else {
			baseline
		}
	}

	/// Returns `true` if `key` is present in any slot (stash, target, or probing).
	/// With skewed distribution: keys with baseline < num_target_buckets can be in target or probing;
	/// keys with baseline >= num_target_buckets can only be in probing or stash.
	pub fn contains_key(&self, key: &K) -> bool {
		let stash_idx = get_index(key, self.stash_size);
		let baseline_idx = get_index(key, self.segment_size);

		if self.stash_buckets[stash_idx].get_position(key).is_some() {
			return true;
		}

		if baseline_idx < self.num_target_buckets {
			// Key naturally belongs to target tier: check its target and probing buckets
			let target_idx = baseline_idx;
			let probing_idx = self.num_target_buckets + (baseline_idx % self.num_probing_buckets);
			self.buckets[target_idx].get_position(key).is_some()
				|| self.buckets[probing_idx].get_position(key).is_some()
		} else {
			// Key naturally belongs to probing tier: only check its probing bucket
			let probing_idx = baseline_idx;
			self.buckets[probing_idx].get_position(key).is_some()
		}
	}

	/// Total items across all normal and stash buckets.
	pub fn total_items(&self) -> usize {
		self.buckets.iter().map(|b| b.size()).sum::<usize>()
			+ self.stash_buckets.iter().map(|b| b.size()).sum::<usize>()
	}

	/// Maximum items the segment can hold: sum of max_size across all normal and stash buckets.
	pub fn max_capacity(&self) -> usize {
		self.buckets.iter().map(|b| b.get_max_size()).sum::<usize>()
			+ self.stash_buckets.iter().map(|b| b.get_max_size()).sum::<usize>()
	}

	/// Returns `true` if every item resides in a bucket consistent with its key's derived slot.
	///
	/// With skewed distribution:
	/// - Keys with baseline < num_target_buckets can be in target [baseline] or probing [num_target_buckets + baseline % num_probing_buckets]
	/// - Keys with baseline >= num_target_buckets can only be in probing [baseline]
	/// - Stash bucket must match stash index of key.
	pub fn check_slot_legality(&self) -> bool {
		for (i, bucket) in self.buckets.iter().enumerate() {
			for item in bucket.get_items() {
				let baseline = get_index(&item.key, self.segment_size);
				let is_legal = if baseline < self.num_target_buckets {
					let target = baseline;
					let probing = self.num_target_buckets + (baseline % self.num_probing_buckets);
					i == target || i == probing
				} else {
					let probing = baseline;
					i == probing
				};
				if !is_legal {
					return false;
				}
			}
		}
		for (j, bucket) in self.stash_buckets.iter().enumerate() {
			for item in bucket.get_items() {
				if j != get_index(&item.key, self.stash_size) {
					return false;
				}
			}
		}
		true
	}

	/// Returns `true` if no individual bucket holds more items than its `max_size`.
	pub fn check_capacity_bound(&self) -> bool {
		self.buckets.iter().all(|b| b.size() <= b.get_max_size())
			&& self.stash_buckets.iter().all(|b| b.size() <= b.get_max_size())
	}

	/// Push `item` directly into the normal bucket at `bucket_idx`, bypassing slot derivation
	/// and capacity limits. Intended only for invariant-suite negative fixtures.
	pub fn force_push_normal_bucket(&mut self, bucket_idx: usize, item: Item<K, V>) {
		self.buckets[bucket_idx].get_items_mut().push(item);
	}

	/// Inserts `item` at `pos` in the bucket identified by `slot` (derived from `item.key`).
	/// Returns the evicted item, if any.
	///
	/// With skewed distribution:
	/// - Target slot for baseline < num_target_buckets routes to bucket [baseline]
	/// - Target slot for baseline >= num_target_buckets routes to bucket [(baseline - num_target_buckets) % num_target_buckets]
	/// - Probing slot for baseline < num_target_buckets routes to bucket [num_target_buckets + baseline % num_probing_buckets]
	/// - Probing slot for baseline >= num_target_buckets routes to bucket [baseline]
	pub fn put_at(&mut self, slot: Slot, pos: InsertPosition, item: Item<K, V>) -> Option<Item<K, V>> {
		let baseline_idx = get_index(&item.key, self.segment_size);
		let bucket = match slot {
			Slot::Stash => {
				let idx = get_index(&item.key, self.stash_size);
				&mut self.stash_buckets[idx]
			}
			Slot::Target => {
				let idx = if baseline_idx < self.num_target_buckets {
					baseline_idx
				} else {
					(baseline_idx - self.num_target_buckets) % self.num_target_buckets
				};
				&mut self.buckets[idx]
			}
			Slot::Probing => {
				let idx = if baseline_idx < self.num_target_buckets {
					self.num_target_buckets + (baseline_idx % self.num_probing_buckets)
				} else {
					baseline_idx
				};
				&mut self.buckets[idx]
			}
		};
		let (_, evicted) = bucket.put_at(item, pos);
		if let Some(ref e) = evicted {
			debug!("SegmentCore.put_at: slot={slot:?} pos={pos:?} evicted_key={:?}", e.key);
		}
		evicted
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;
	use crate::shared::traits::bucket::InsertPosition;
	use crate::test_utils::{find_keys_for_bucket, small_dash_settings};

	#[test]
	fn try_new_rejects_single_normal_bucket() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_normal_buckets = 1;
		assert!(s.validate().is_err());
		assert!(SegmentCore::<i64, i64>::try_new(&s).is_none());
	}

	#[test]
	fn try_new_rejects_zero_stash_buckets() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 0;
		assert!(s.validate().is_err());
		assert!(SegmentCore::<i64, i64>::try_new(&s).is_none());
	}

	// ── total_items ──────────────────────────────────────────────────────────

	#[test]
	fn total_items_empty() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let core = SegmentCore::<i64, i64>::new(&s);
		assert_eq!(core.total_items(), 0);
	}

	#[test]
	fn total_items_counts_normal_and_stash() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let normal_key = find_keys_for_bucket(0, s.num_of_normal_buckets, 1)[0];
		core.put_at(Slot::Target, InsertPosition::Back, Item::new(normal_key, normal_key));
		assert_eq!(core.total_items(), 1);
		let stash_key = find_keys_for_bucket(0, s.num_of_stash_buckets, 1)[0];
		core.put_at(Slot::Stash, InsertPosition::Back, Item::new(stash_key, stash_key));
		assert_eq!(core.total_items(), 2);
		// Verify the stash bucket actually received the item, not just the count.
		assert!(core.stash_buckets[0].get_position(&stash_key).is_some());
	}

	// ── max_capacity ─────────────────────────────────────────────────────────

	#[test]
	fn max_capacity_matches_settings() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let core = SegmentCore::<i64, i64>::new(&s);
		let expected = (s.num_of_normal_buckets + s.num_of_stash_buckets) * s.bucket_size;
		assert_eq!(core.max_capacity(), expected);
	}

	// ── check_slot_legality ──────────────────────────────────────────────────

	#[test]
	fn check_slot_legality_empty_is_legal() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		assert!(SegmentCore::<i64, i64>::new(&s).check_slot_legality());
	}

	#[test]
	fn check_slot_legality_target_slot_is_legal() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let key = find_keys_for_bucket(1, s.num_of_normal_buckets, 1)[0];
		core.put_at(Slot::Target, InsertPosition::Back, Item::new(key, key));
		assert!(core.check_slot_legality());
	}

	#[test]
	fn check_slot_legality_probing_slot_is_legal() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// A key at target bucket 0 placed in its probing bucket 1 must be legal.
		let key = find_keys_for_bucket(0, s.num_of_normal_buckets, 1)[0];
		core.put_at(Slot::Probing, InsertPosition::Back, Item::new(key, key));
		assert!(core.check_slot_legality());
	}

	#[test]
	fn check_slot_legality_probing_wraps_around() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// A key at target bucket 3 (last) has probing index 0 via modulo wrap.
		// Placing it in bucket 0 via Slot::Probing must be legal.
		let key = find_keys_for_bucket(s.num_of_normal_buckets - 1, s.num_of_normal_buckets, 1)[0];
		core.put_at(Slot::Probing, InsertPosition::Back, Item::new(key, key));
		assert!(core.check_slot_legality());
	}

	#[test]
	fn check_slot_legality_detects_wrong_bucket() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// Key hashes to bucket 2; forcing it into bucket 0 violates legality.
		let bad_key = find_keys_for_bucket(2, s.num_of_normal_buckets, 1)[0];
		core.force_push_normal_bucket(0, Item::new(bad_key, bad_key));
		assert!(!core.check_slot_legality());
	}

	#[test]
	fn check_slot_legality_detects_wrong_stash_bucket() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// Find a key that hashes to stash 1, then force it into stash 0.
		let key = (0i64..200)
			.find(|k| get_index(k, s.num_of_stash_buckets) == 1)
			.expect("no key hashes to stash 1");
		core.stash_buckets[0].get_items_mut().push(Item::new(key, key));
		assert!(!core.check_slot_legality());
	}

	// ── check_capacity_bound ─────────────────────────────────────────────────

	#[test]
	fn check_capacity_bound_empty_is_within_bound() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		assert!(SegmentCore::<i64, i64>::new(&s).check_capacity_bound());
	}

	#[test]
	fn check_capacity_bound_at_limit_is_within_bound() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let keys = find_keys_for_bucket(0, s.num_of_normal_buckets, s.bucket_size);
		for k in keys {
			core.force_push_normal_bucket(0, Item::new(k, k));
		}
		assert!(core.check_capacity_bound());
	}

	#[test]
	fn check_capacity_bound_detects_normal_overflow() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// Push bucket_size + 1 items to exceed capacity.
		let keys = find_keys_for_bucket(0, s.num_of_normal_buckets, s.bucket_size + 1);
		for k in keys {
			core.force_push_normal_bucket(0, Item::new(k, k));
		}
		assert!(!core.check_capacity_bound());
	}

	#[test]
	fn check_capacity_bound_detects_stash_overflow() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// Overfill stash bucket 0 beyond its max_size.
		let keys = find_keys_for_bucket(0, s.num_of_stash_buckets, s.bucket_size + 1);
		for k in keys {
			core.stash_buckets[0].get_items_mut().push(Item::new(k, k));
		}
		assert!(!core.check_capacity_bound());
	}

	// ── force_push_normal_bucket ─────────────────────────────────────────────

	#[test]
	fn force_push_normal_bucket_item_appears_in_target_bucket() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// Use a key whose target is bucket 0, not bucket 3, to confirm the push
		// bypasses slot derivation and lands in the requested bucket regardless.
		let key = find_keys_for_bucket(0, s.num_of_normal_buckets, 1)[0];
		core.force_push_normal_bucket(3, Item::new(key, key));
		assert!(core.buckets[3].get_position(&key).is_some());
		assert_eq!(core.total_items(), 1);
	}
}
