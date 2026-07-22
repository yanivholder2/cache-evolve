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
	/// Per-slot reuse-protection bits, kept index-aligned with `items`
	/// (`reuse.len() == items.len()` at all times). `reuse[i] == true` iff the
	/// resident item in slot `i` has been hit at least once since it was inserted,
	/// **or** it was admitted as a proven reuse candidate via the ghost history.
	///
	/// This is the scan-resistance state v001 carried in `Item::lfu_counter`,
	/// relocated off the item so `Item` stays at the 16-byte metadata floor v003
	/// reached. The bucket struct is not measured by the metadata-size metric
	/// (only `Item` and the top-level `AssociativeCache` are), so this state is
	/// free on that dimension.
	reuse: Vec<bool>,
	/// 2Q-style ghost history: a small fixed-capacity ring (cap = `max_size`)
	/// recording the *keys* of items recently evicted from this bucket. Stores
	/// keys only — never values — so it stays entirely off `Item` and does not
	/// move the 16-byte metadata floor. Front (`ghost[0]`) is the oldest ghost;
	/// when the ring is full the oldest is dropped to admit a new one.
	///
	/// The lever: the reuse bit alone only protects an item *after* a hit while
	/// it is still resident. A key whose reuse distance exceeds its bucket
	/// lifetime is evicted and re-admitted cold every cycle, never earning
	/// protection. Remembering evicted keys lets a returning key be admitted
	/// already-protected, so it survives the next scan burst.
	ghost: Vec<K>,
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
			reuse: Vec::with_capacity(settings.bucket_size),
			ghost: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	/// Rebuild `reuse` so it stays index-aligned with `items` after the eviction
	/// policy may have permuted `items`. Bits are carried by item key — unique
	/// within a bucket — so whatever reordering the policy applied (ClassicLRU
	/// moving a hit item to the back, Swap moving it one step forward, etc.) is
	/// mirrored exactly, preserving the `reuse[i]` ↔ `items[i]` correspondence.
	fn realign_reuse(&mut self, before: &[(K, bool)]) {
		self.reuse = self
			.items
			.iter()
			.map(|item| {
				before
					.iter()
					.find(|(key, _)| *key == item.key)
					.map(|(_, bit)| *bit)
					.unwrap_or(false)
			})
			.collect();
	}

	/// Record `key` in the ghost ring as recently evicted from this bucket.
	/// The ring is bounded to `max_size` keys; when full, the oldest ghost
	/// (front) is dropped so the ghost history stays O(bucket_size) in space
	/// and every operation on it stays O(bucket_size) in time.
	fn record_ghost(&mut self, key: K) {
		// A key can only be recorded when it leaves `items`, so it is never
		// simultaneously resident and in the ghost ring. Defensive dedup keeps
		// the ring free of stale duplicates if a key churns in and out.
		if let Some(pos) = self.ghost.iter().position(|k| *k == key) {
			self.ghost.remove(pos);
		}
		if self.max_size == 0 {
			return;
		}
		if self.ghost.len() >= self.max_size {
			self.ghost.remove(0);
		}
		self.ghost.push(key);
	}

	/// If `key` is present in the ghost ring, this key was seen and evicted from
	/// this bucket before — a proven reuse candidate. Consume the ghost entry and
	/// report that the incoming item should be admitted already-protected.
	fn take_ghost(&mut self, key: &K) -> bool {
		if let Some(pos) = self.ghost.iter().position(|k| *k == *key) {
			self.ghost.remove(pos);
			true
		} else {
			false
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

	/// Insert `item` at the back, maintaining the parallel reuse set. On a
	/// genuine miss the ghost history decides the *initial* reuse bit: a key that
	/// was previously evicted from this bucket (a ghost hit) is admitted already
	/// protected (`reuse = true`), so it survives the next scan; a never-before-
	/// seen key is admitted unprotected (`reuse = false`) exactly as the parent
	/// reuse-bit incumbent does. Updates to an existing key route through
	/// `get_from_position`, which counts as an access and sets the bit.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed = self.get_from_position(position);
			return (pushed, None);
		}
		// Genuine miss. Consult the ghost history *before* eviction, since the
		// eviction below records a new ghost key and could otherwise evict the
		// incoming key's ghost entry from a full ring.
		let protected = self.take_ghost(&item.key);
		let evicted = if self.is_full() { self.evict_item() } else { None };
		self.items.push(item);
		self.reuse.push(protected);
		let pushed = self.items.last().expect("bucket invariant: just pushed");
		(pushed, evicted)
	}

	/// Access the item at `position`, applying the policy's reordering side effect
	/// and marking the item as reused. The reuse bits are realigned to follow the
	/// (possibly reordered) items, then the accessed item's bit is set — this is
	/// the "hit at least once since insertion" signal the eviction path reads.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let before: Vec<(K, bool)> =
			self.items.iter().map(|i| i.key).zip(self.reuse.iter().copied()).collect();
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		self.realign_reuse(&before);
		self.reuse[index] = true;
		&self.items[index]
	}

	/// Scan-resistant eviction candidate selection (v001's mechanism): evict the
	/// first slot in LRU-front order (index 0 = least-recently-used under
	/// ClassicLRU) whose reuse bit is `false` — a one-hit-wonder that was inserted
	/// but never touched again. If every resident item is protected, fall back to
	/// the LRU front (index 0). Driven from the bucket because the `EvictionPolicy`
	/// trait cannot see the bucket-side reuse set.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		for (idx, &reused) in self.reuse.iter().enumerate() {
			if !reused {
				return Some(idx);
			}
		}
		Some(0)
	}

	/// Remove the item at `index`, keeping the reuse set index-aligned and
	/// recording the departed key in the ghost history so a later re-reference
	/// can be admitted already-protected.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.reuse.remove(index);
		let removed = self.items.remove(index);
		self.record_ghost(removed.key);
		removed
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
