use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::trace;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Newly admitted items are inserted at `len / INSERT_FRACTION_DIV` (measured
/// from the LRU end, index 0) instead of the MRU tail. LRU-insertion-point
/// scheme: a new item must be hit again before it drifts the remaining
/// ~1/INSERT_FRACTION_DIV of the bucket down to the eviction end.
const INSERT_FRACTION_DIV: usize = 4;

/// Per-bucket saturating feedback counter (DIP-style PSEL) scoring MRU
/// sampling by the fate of the items it chose.
const PSEL_MAX: u8 = 15;
const PSEL_INIT: u8 = 8;
const PSEL_THRESHOLD: u8 = 8;
/// MRU-insert period while MRU sampling is paying off (psel >= threshold).
const PERIOD_BIP: u32 = 16;
/// MRU-insert period while it is not; non-zero so the bucket keeps learning.
const PERIOD_LIP: u32 = 128;
/// Number of recently MRU-sampled keys tracked per bucket.
const SAMPLE_SLOTS: usize = 8;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Per-bucket count of new-item insertions (drives the bimodal choice).
	insert_count: u32,
	/// Saturating feedback counter in `0..=PSEL_MAX`.
	psel: u8,
	/// Ring of keys recently inserted at the MRU tail and not yet resolved.
	sampled: [Option<K>; SAMPLE_SLOTS],
	/// Ring write cursor into `sampled`.
	sample_next: usize,
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
			insert_count: 0,
			psel: PSEL_INIT,
			sampled: [None; SAMPLE_SLOTS],
			sample_next: 0,
		}
	}

	/// If `key` is a tracked MRU sample, clears its slot and returns true.
	fn take_sampled(&mut self, key: &K) -> bool {
		for slot in self.sampled.iter_mut() {
			if slot.as_ref() == Some(key) {
				*slot = None;
				return true;
			}
		}
		false
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

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let key = self.items[position].key;
		if self.take_sampled(&key) {
			// An MRU-sampled item was reused: reward MRU sampling.
			self.psel = (self.psel + 1).min(PSEL_MAX);
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	/// Adaptive bimodal LRU-insertion-point put: hits on existing keys behave
	/// as the trait default (policy-driven promotion, with PSEL feedback in
	/// `get_from_position`); new items are inserted at the bucket's 25%
	/// position (from the LRU end), except every `period`-th new item, which
	/// goes to the MRU tail and is tracked as a sample. `period` is
	/// `PERIOD_BIP` or `PERIOD_LIP` depending on the bucket's PSEL.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		if let Some(ref evicted) = evicted_item {
			if self.take_sampled(&evicted.key) {
				// An MRU-sampled item died without reuse: penalise sampling.
				self.psel = self.psel.saturating_sub(1);
			}
		}
		self.insert_count = self.insert_count.wrapping_add(1);
		let period = if self.psel >= PSEL_THRESHOLD { PERIOD_BIP } else { PERIOD_LIP };
		let idx = if self.insert_count % period == 0 {
			self.sampled[self.sample_next] = Some(item.key);
			self.sample_next = (self.sample_next + 1) % SAMPLE_SLOTS;
			self.items.push(item);
			self.items.len() - 1
		} else {
			let ins = self.items.len() / INSERT_FRACTION_DIV;
			if ins == 0 {
				self.items.push(item);
				self.items.len() - 1
			} else {
				self.items.insert(ins, item);
				ins
			}
		};
		(&self.items[idx], evicted_item)
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
