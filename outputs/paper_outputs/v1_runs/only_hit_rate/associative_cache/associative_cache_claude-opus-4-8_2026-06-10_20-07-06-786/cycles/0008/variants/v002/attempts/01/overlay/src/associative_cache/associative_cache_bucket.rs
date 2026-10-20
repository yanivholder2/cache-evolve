use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
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
		&mut self.items
	}

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	// ---- Segmented (three-tier SLRU-style) bucket overrides ---------------------
	//
	// The bucket is a single recency-ordered `items` vector (front = LRU end,
	// back = MRU end). Each item's `lfu_counter` records its retention *tier*:
	//   * lfu_counter == 0 -> probationary (new keys; evicted first)
	//   * lfu_counter == 1 -> protected (proven hot once)
	//   * lfu_counter == 2 -> elite (proven hot more than once; H02)
	//
	// New keys are admitted to the probationary front (index 0). A hit bumps the
	// item's tier toward the elite cap of 2 and moves it to the MRU back.
	// Eviction (see `eviction_candidate_index`) sacrifices the oldest
	// probationary item first; only once every resident item is protected does it
	// reach into the protected set, and there it discards the *least* protected
	// (counter == 1 before counter == 2), LRU-first within that tier. This is
	// protection-additive: it never raises the bar to reach protection
	// (promote-on-first-hit) and never demotes — it only reorders eviction
	// priority among already-protected keys so the hottest survive one extra
	// round of pressure.

	/// Inserts a *new* key at the probationary front (index 0), evicting one item
	/// first if the bucket is full. Existing keys update in place per policy.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		// Admit to the probationary front. `Item::new` already sets lfu_counter = 0.
		self.items.insert(0, item);
		(&self.items[0], evicted_item)
	}

	/// On a hit: bump the access tier toward the elite cap of 2 (0 -> 1 grants
	/// protection, 1 -> 2 earns elite, 2 stays capped) and move the item to the
	/// MRU back.
	///
	/// Fast path: an item already at the MRU back needs no memmove — the
	/// remove+push below would restore a byte-identical vector — so it is returned
	/// in place. The tier bump is applied *before* this early return so that the
	/// hottest (already-MRU) key can still earn elite status on an at-back hit.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		// Bump the tier toward the elite cap (2). This is the only mutation a hit
		// makes to the counter, and it is protection-additive: never demotes.
		if self.items[position].lfu_counter < 2 {
			self.items[position].lfu_counter += 1;
		}
		if position == self.items.len() - 1 {
			return &self.items[position];
		}
		let item = self.items.remove(position);
		self.items.push(item);
		self.items.last().expect("bucket invariant: just pushed")
	}

	/// Choose the eviction victim by tier, then recency:
	///
	/// 1. If any *probationary* item (`lfu_counter == 0`) is resident, evict the
	///    oldest one — the highest-index counter-0 item. New keys insert at the
	///    front, so older un-promoted keys drift to higher indices; the highest
	///    such index is the probationary LRU.
	/// 2. Otherwise every resident item is protected. Evict the *least* protected
	///    item: the minimum `lfu_counter` among residents (prefer a counter == 1
	///    victim over a counter == 2 victim), breaking ties by lowest index (the
	///    protected LRU end).
	///
	/// A small linear scan is acceptable here — hit-rate is the only scored
	/// dimension — so the parent's strict O(1) contiguous-region arithmetic is
	/// relaxed in favour of an exact tiered choice (H02).
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		// Tier 0: oldest probationary = highest index with lfu_counter == 0.
		if let Some(idx) = self
			.items
			.iter()
			.enumerate()
			.rev()
			.find(|(_, it)| it.lfu_counter == 0)
			.map(|(i, _)| i)
		{
			return Some(idx);
		}
		// All protected: evict the least-protected, LRU-first within that tier.
		let min_counter = self.items.iter().map(|it| it.lfu_counter).min()?;
		self.items.iter().position(|it| it.lfu_counter == min_counter)
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
