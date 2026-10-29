use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::{debug, trace};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Bucket container. When the configured policy is `ClassicLRU`, the bucket
/// runs an in-place CLOCK approximation instead of move-to-back: a hit sets
/// the item's reference bit (`lfu_counter = 1`) without reordering, and an
/// insert into a full bucket sweeps a per-bucket hand, clearing set bits until
/// it finds an unreferenced slot, which is replaced in place. All other
/// policies take the original code paths unchanged.
///
/// Lookup goes through `keys`, a dense mirror of the item keys. Invariant
/// (when `keys_dirty` is false): `keys.len() == items.len()` and
/// `keys[i] == items[i].key`. Scanning 8-byte keys touches far fewer cache
/// lines than striding through `Item`s. Handing out `get_items_mut` marks the
/// mirror dirty. While it is dirty, lookups fall back to scanning `items`, and
/// the next mutating operation rebuilds the mirror.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Dense parallel key array: `keys[i] == items[i].key`.
	keys: Vec<K>,
	/// Set when `items` may have been mutated externally via `get_items_mut`.
	keys_dirty: bool,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// CLOCK hand: index of the next slot to inspect on eviction.
	hand: usize,
	/// Cached `eviction_policy.name() == "ClassicLRU"`.
	clock: bool,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let eviction_policy: Box<dyn EvictionPolicy<K, V>> = settings.eviction_policy.build();
		let clock = eviction_policy.name() == "ClassicLRU";
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			keys: Vec::with_capacity(settings.bucket_size),
			keys_dirty: false,
			max_size: settings.bucket_size,
			eviction_policy,
			hand: 0,
			clock,
		}
	}

	/// Rebuilds `keys` from `items` unconditionally.
	#[inline(never)]
	fn rebuild_keys(&mut self) {
		self.keys.clear();
		self.keys.extend(self.items.iter().map(|it| it.key));
		self.keys_dirty = false;
	}

	/// Restores the key-mirror invariant if `items` may have changed externally.
	#[inline]
	fn sync_keys(&mut self) {
		if self.keys_dirty {
			self.rebuild_keys();
		}
	}

	/// Pushes an item, keeping `keys` in sync.
	#[inline]
	fn push_item(&mut self, item: Item<K, V>) {
		let key = item.key;
		self.items.push(item);
		self.keys.push(key);
	}

	/// CLOCK sweep: clears reference bits starting at `hand` until an
	/// unreferenced slot is found; returns its index and leaves `hand`
	/// pointing at it. Precondition: `items` non-empty. Terminates within two
	/// passes since the first pass clears every bit it skips.
	#[inline]
	fn clock_sweep(&mut self) -> usize {
		let len = self.items.len();
		if self.hand >= len {
			self.hand = 0;
		}
		loop {
			let slot = &mut self.items[self.hand];
			if slot.lfu_counter != 0 {
				slot.lfu_counter = 0;
				self.hand += 1;
				if self.hand == len {
					self.hand = 0;
				}
			} else {
				return self.hand;
			}
		}
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
		// Callers may reorder/insert/remove; the mirror is rebuilt lazily.
		self.keys_dirty = true;
		&mut self.items
	}

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	#[inline]
	fn get_position(&self, key: &K) -> Option<usize> {
		if self.keys_dirty {
			return self.items.iter().position(|d| d.key == *key);
		}
		debug_assert_eq!(self.keys.len(), self.items.len());
		self.keys.iter().position(|k| k == key)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.clock {
			self.items[position].lfu_counter = 1;
			return &self.items[position];
		}
		self.sync_keys();
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		// Non-clock policies may reorder items arbitrarily (move-to-back, swap,
		// ...). Rebuild the mirror on this path: correctness over speed.
		self.rebuild_keys();
		&self.items[index]
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		self.sync_keys();
		if !self.clock {
			// Original default `Bucket::put` behaviour.
			if let Some(position) = self.get_position(&item.key) {
				trace!("Bucket.put: key={:?} update_existing", item.key);
				self.items[position].value = item.value;
				let pushed_item = self.get_from_position(position);
				return (pushed_item, None);
			}
			let evicted_item = if self.is_full() { self.evict_item() } else { None };
			self.push_item(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, evicted_item);
		}

		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			let slot = &mut self.items[position];
			slot.value = item.value;
			slot.lfu_counter = 1;
			return (&self.items[position], None);
		}

		let mut item = item;
		item.lfu_counter = 0;

		if self.items.len() < self.max_size {
			self.push_item(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, None);
		}

		if self.items.is_empty() {
			// max_size == 0: nothing can be stored; mirror the default path's
			// behaviour of no eviction (validation forbids this geometry anyway).
			self.push_item(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, None);
		}

		let victim_idx = self.clock_sweep();
		let new_key = item.key;
		let old = std::mem::replace(&mut self.items[victim_idx], item);
		self.keys[victim_idx] = new_key;
		self.hand = victim_idx + 1;
		if self.hand == self.items.len() {
			self.hand = 0;
		}
		debug!("Bucket.put: policy=CLOCK evicted_key={:?}", old.key);
		(&self.items[victim_idx], Some(old))
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		self.sync_keys();
		if !self.clock {
			let idx = self.eviction_candidate_index()?;
			let evicted = self.remove_at(idx);
			debug!(
				"Bucket.evict_item: policy={} evicted_key={:?}",
				self.eviction_policy.name(),
				evicted.key
			);
			return Some(evicted);
		}
		if self.items.is_empty() {
			return None;
		}
		let idx = self.clock_sweep();
		let evicted = self.remove_at(idx);
		if idx < self.hand {
			self.hand -= 1;
		}
		if self.hand >= self.items.len() {
			self.hand = 0;
		}
		debug!("Bucket.evict_item: policy=CLOCK evicted_key={:?}", evicted.key);
		Some(evicted)
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		if !self.clock {
			return Some(self.eviction_policy.eviction_candidate_index(&self.items));
		}
		let len = self.items.len();
		let start = if self.hand >= len { 0 } else { self.hand };
		for step in 0..len {
			let idx = (start + step) % len;
			if self.items[idx].lfu_counter == 0 {
				return Some(idx);
			}
		}
		Some(start)
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.sync_keys();
		self.keys.remove(index);
		self.items.remove(index)
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
