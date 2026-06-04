use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// An associative cache bucket that combines LFU-weighted ClassicLRU eviction
/// with a per-bucket second-chance stash and a key-only ghost tier.
///
/// Items evicted from the primary buffer are placed into a bounded FIFO stash
/// rather than discarded. On a subsequent `get` or `put` for a stash-resident
/// key the item is promoted back to primary with an lfu_counter credit of +2,
/// giving it enough of a head start to survive the next cold-burst insertion
/// cycle rather than being immediately re-evicted.
///
/// Keys displaced from the stash by FIFO overflow enter a key-only ghost tier.
/// When a ghost key is later inserted as a brand-new put, its lfu_counter is
/// initialized to 1, giving it a mild head start that reduces the probability
/// of immediate re-eviction without crowding out genuinely hot items.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Second-chance stash: items evicted from primary live here until either
	/// promoted on a hit or displaced by FIFO overflow.
	stash: Vec<Item<K, V>>,
	/// Maximum number of items the stash may hold.
	stash_max_size: usize,
	/// Key-only ghost tier: keys displaced from the stash by FIFO overflow.
	/// Holds only keys (no values) so it provides extended eviction memory
	/// at a fraction of the byte cost of primary or stash entries.
	ghost: Vec<K>,
	/// Maximum number of keys the ghost may hold (equals primary capacity).
	ghost_max_size: usize,
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
			stash: Vec::new(),
			stash_max_size: (settings.bucket_size / 4).max(1),
			ghost: Vec::new(),
			ghost_max_size: settings.bucket_size,
		}
	}

	/// Append `key` to the ghost tier, enforcing the FIFO size cap.
	/// If the ghost is at capacity the oldest entry (index 0) is dropped first.
	fn ghost_push(&mut self, key: K) {
		if self.ghost.len() >= self.ghost_max_size {
			self.ghost.remove(0);
		}
		self.ghost.push(key);
	}

	/// Append `item` to the stash, enforcing the FIFO size cap.
	/// If the stash is at capacity the oldest entry (index 0) is displaced and
	/// its key is forwarded to the ghost tier before being dropped.
	fn stash_push(&mut self, item: Item<K, V>) {
		if self.stash.len() >= self.stash_max_size {
			let displaced = self.stash.remove(0);
			self.ghost_push(displaced.key);
		}
		self.stash.push(item);
	}

	/// Evict the LFU candidate from the primary buffer into the stash.
	/// No-op when primary is empty.
	fn evict_to_stash(&mut self) {
		if self.items.is_empty() {
			return;
		}
		let idx = self.eviction_policy.eviction_candidate_index(&self.items);
		let evicted = self.items.remove(idx);
		self.stash_push(evicted);
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

	/// Insert or update a key-value pair.
	///
	/// Three cases in priority order:
	/// 1. Key already in primary: update value in place, apply policy touch.
	/// 2. Key found in stash: promote to primary (evicting one primary item to
	///    stash first if needed), credit two lfu_counter ticks so the promoted
	///    item survives at least one more eviction cycle before being re-evicted.
	/// 3. New key: evict one primary item to stash if needed, then push.
	///    If the key is found in the ghost tier, initialize lfu_counter to 1
	///    to give it a mild head start — it completed the primary→stash→ghost
	///    journey and is more likely to be accessed again than a truly cold key.
	///
	/// The second return slot is always `None` because evicted items go to the
	/// stash rather than being discarded; the framework never observes them as
	/// removed until they age out of the stash.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Case 1: key already resident in primary
		if let Some(position) = self.get_position(&item.key) {
			self.get_items_mut()[position].value = item.value;
			let new_pos = self.get_from_position(position);
			return (new_pos, None);
		}

		// Case 2: key found in stash — promote it with +2 credit
		if let Some(stash_pos) = self.stash.iter().position(|i| i.key == item.key) {
			let mut stash_item = self.stash.remove(stash_pos);
			stash_item.lfu_counter += 2;
			if self.items.len() == self.max_size {
				self.evict_to_stash();
			}
			self.items.push(stash_item);
			let back = self.items.len() - 1;
			return (&self.items[back], None);
		}

		// Case 3: brand-new key
		if self.items.len() == self.max_size {
			self.evict_to_stash();
		}
		let mut new_item = item;
		if let Some(ghost_pos) = self.ghost.iter().position(|k| *k == new_item.key) {
			self.ghost.remove(ghost_pos);
			new_item.lfu_counter = 1; // mild head start: completed the primary→stash→ghost journey
		}
		self.items.push(new_item);
		let back = self.items.len() - 1;
		(&self.items[back], None)
	}

	/// Look up a key.
	///
	/// Checks primary first; on a miss checks the stash. A stash hit promotes
	/// the item to primary (with two lfu_counter credits) before returning it,
	/// so the promoted item has enough frequency head-start to survive the next
	/// cold-burst insertion cycle rather than being immediately re-evicted.
	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		// Primary hit
		if let Some(position) = self.get_position(key) {
			return Some(self.get_from_position(position));
		}

		// Stash hit: promote to primary with +2 credit
		if let Some(stash_pos) = self.stash.iter().position(|i| &i.key == key) {
			let mut stash_item = self.stash.remove(stash_pos);
			stash_item.lfu_counter += 2;
			if self.items.len() == self.max_size {
				self.evict_to_stash();
			}
			self.items.push(stash_item);
			let back = self.items.len() - 1;
			return Some(&self.items[back]);
		}

		None
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
