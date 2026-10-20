use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Maximum 2-bit re-reference prediction value. A slot at this value is the victim candidate.
const RRPV_MAX: u64 = 3;
/// RRPV assigned on insert: a long (not maximal) re-reference interval.
const RRPV_INSERT: u64 = 2;
/// Number of 2-bit RRPV fields packed into one `u64` word.
const SLOTS_PER_WORD: usize = 32;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Per-slot 2-bit RRPV, packed 32 slots per `u64`. Slot `i` lives in word
	/// `i / 32` at bit offset `2 * (i % 32)`. Buckets of up to 32 slots use one word.
	rrpv: Vec<u64>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			rrpv: vec![0; (settings.bucket_size + SLOTS_PER_WORD - 1) / SLOTS_PER_WORD],
		}
	}

	fn rrpv_get(&self, slot: usize) -> u64 {
		(self.rrpv[slot / SLOTS_PER_WORD] >> (2 * (slot % SLOTS_PER_WORD))) & RRPV_MAX
	}

	fn rrpv_set(&mut self, slot: usize, value: u64) {
		let shift = 2 * (slot % SLOTS_PER_WORD);
		let word = &mut self.rrpv[slot / SLOTS_PER_WORD];
		*word = (*word & !(RRPV_MAX << shift)) | ((value & RRPV_MAX) << shift);
	}

	/// Chooses the victim: the first slot (in slot order) with RRPV == 3. Equivalent
	/// to repeated single-step aging (each round adds 1 to every slot, saturating at 3
	/// and stopping once some slot reaches 3), computed as one max-scan followed by
	/// one bulk add of `delta = RRPV_MAX - max_rrpv`, saturating at `RRPV_MAX`.
	fn select_victim(&mut self) -> usize {
		let len = self.items.len();
		let mut max_rrpv = 0;
		for s in 0..len {
			max_rrpv = max_rrpv.max(self.rrpv_get(s));
		}
		let delta = RRPV_MAX - max_rrpv;
		if delta > 0 {
			for s in 0..len {
				let v = self.rrpv_get(s);
				self.rrpv_set(s, (v + delta).min(RRPV_MAX));
			}
		}
		(0..len)
			.find(|&s| self.rrpv_get(s) == RRPV_MAX)
			.expect("select_victim invariant: some slot reaches RRPV_MAX after bulk aging")
	}

	/// Drops the RRPV of `slot`, shifting higher slots down to mirror `Vec::remove`.
	/// `old_len` is the item count before the removal.
	fn rrpv_remove(&mut self, slot: usize, old_len: usize) {
		for s in slot..old_len - 1 {
			let next = self.rrpv_get(s + 1);
			self.rrpv_set(s, next);
		}
		self.rrpv_set(old_len - 1, 0);
	}
}

impl<K, V> Bucket<K, V> for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		&mut self.items
	}

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	/// Insert with RRPV bookkeeping. Existing keys update in place and count as a hit.
	/// New items enter at the back with RRPV = 2 and no items are reordered.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		let slot = self.items.len();
		self.rrpv_set(slot, RRPV_INSERT);
		self.items.push(item);
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
	}

	/// Evicts the RRPV-selected victim and drops its RRPV slot.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}
		let old_len = self.items.len();
		let slot = self.select_victim();
		self.rrpv_remove(slot, old_len);
		Some(self.remove_at(slot))
	}

	/// A get hit sets the slot's RRPV to 0. Items never move within the bucket.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		self.rrpv_set(position, 0);
		&self.items[position]
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for item in &self.items {
			writeln!(f, "    {}", item)?;
		}
		Ok(())
	}
}
