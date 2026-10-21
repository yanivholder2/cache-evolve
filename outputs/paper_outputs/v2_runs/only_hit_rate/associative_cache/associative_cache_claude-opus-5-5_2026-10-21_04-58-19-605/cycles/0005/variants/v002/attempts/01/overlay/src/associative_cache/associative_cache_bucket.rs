use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Insertion depth for new (missed) keys under ClassicLRU, as a fraction of the
/// bucket length measured from the eviction end (index 0). 1/8 = an eighth of
/// the way up from the eviction end (close to, but not quite, pure LIP).
const INSERT_DEPTH_NUM: usize = 1;
const INSERT_DEPTH_DEN: usize = 8;

/// Insertion depth for keys found in the per-bucket ghost ring (recently
/// evicted from this bucket). Capped reward: 1/4 instead of 1/8.
const GHOST_DEPTH_NUM: usize = 1;
const GHOST_DEPTH_DEN: usize = 4;

/// Number of recently evicted keys remembered per bucket.
const GHOST_LEN: usize = 4;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// When true, new keys are inserted at INSERT_DEPTH_NUM/INSERT_DEPTH_DEN of
	/// the bucket instead of the MRU tail (LIP/BIP-style). Only enabled for ClassicLRU.
	midpoint_insert: bool,
	/// Ring of keys recently evicted on the miss path (midpoint_insert only).
	ghosts: [Option<K>; GHOST_LEN],
	/// Next write slot in `ghosts` (overwrites the oldest entry).
	ghost_pos: u8,
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
			midpoint_insert: settings.eviction_policy == EvictionPolicyKind::ClassicLRU,
			ghosts: [None; GHOST_LEN],
			ghost_pos: 0,
		}
	}

	/// Record an evicted key, overwriting the oldest ghost entry.
	fn record_ghost(&mut self, key: K) {
		self.ghosts[self.ghost_pos as usize] = Some(key);
		self.ghost_pos = ((self.ghost_pos as usize + 1) % GHOST_LEN) as u8;
	}

	/// Linear scan of the ghost ring. On a match, clears the entry and returns true.
	fn in_ghosts(&mut self, key: &K) -> bool {
		for slot in self.ghosts.iter_mut() {
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
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing key: baseline behaviour (update value, promote per policy).
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		if !self.midpoint_insert {
			// Baseline miss path: evict if full, push to the tail.
			let evicted_item = if self.is_full() { self.evict_item() } else { None };
			self.items.push(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, evicted_item);
		}

		// Ghost lookup runs before the eviction, so a key cannot match the key
		// it is about to displace.
		let ghost_hit = self.in_ghosts(&item.key);

		// Capacity check runs before the insert, so size never exceeds max_size.
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		if let Some(ev) = evicted_item.as_ref() {
			self.record_ghost(ev.key);
		}

		// Depth insertion: computed after the eviction. With a full 16-slot
		// bucket, len = 15 here, so idx = 1 (ordinary) or 3 (ghost hit).
		let len = self.items.len();
		let idx = if ghost_hit {
			len * GHOST_DEPTH_NUM / GHOST_DEPTH_DEN
		} else {
			len * INSERT_DEPTH_NUM / INSERT_DEPTH_DEN
		};
		self.items.insert(idx, item);
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
