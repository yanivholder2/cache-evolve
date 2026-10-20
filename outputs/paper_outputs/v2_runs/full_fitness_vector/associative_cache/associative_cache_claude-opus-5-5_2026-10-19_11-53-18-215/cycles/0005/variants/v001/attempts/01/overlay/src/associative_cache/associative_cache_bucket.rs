//! H01 (cycle 0005): tag-first lookup on v003's ghost-guided SLRU bucket.
//!
//! One physical bucket still merges a pair of logical buckets
//! (`2 * bucket_size` slots when paired, `bucket_size` for the trailing
//! unpaired bucket), backed by one contiguous `Vec` split at `len0`:
//!
//! - `items[0..len0]` is **probation** (index 0 = LRU, `len0 - 1` = MRU).
//! - `items[len0..]` is **protected** (index `len0` = LRU, last = MRU).
//!
//! A parallel `tags: Vec<u16>` holds the 16-bit fingerprint of each item
//! (`tags[i]` is the tag of `items[i]` at all times). Lookups scan the 2-byte
//! tag array first (a whole 16-slot run fits in one cache line) and only
//! compare full keys on a tag match. The caller hashes the key once and passes
//! the tag in, so the put path no longer computes a second hash for the ghost
//! ring; evicted items' tags are recorded straight from `tags[0]`.
//!
//! SLRU semantics are exactly v003's:
//! - get, protected hit: rotate to protected MRU.
//! - get, probation hit: promote to protected MRU; demote protected LRU to
//!   probation MRU if protected exceeds `protected_cap` (3/4 of slots).
//! - put (new key): ghost hit → probation MRU; cold key → probation middle
//!   `len0 / 2` (one `mem::replace` + short rotate when full); `len0 == 0` →
//!   v001 behaviour. Every probation eviction records its tag in the ring.
//!
//! SLRU applies only when the configured policy is `ClassicLRU`. Any other
//! policy uses a single-segment fallback through the boxed policy, with
//! `len0 == items.len()`.
//!
//! Note: `Bucket::get_items_mut` still hands out the raw items `Vec`; callers
//! that reorder or resize it through that handle bypass tag maintenance.

use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use crate::shared::utils::hash;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// 16-bit tag from a precomputed key hash: upper 16 bits of the hash, with 0
/// mapped to 1 so that 0 marks an empty ghost slot.
#[inline]
pub(crate) fn tag_of_hash(h: usize) -> u16 {
	(((h as u64) >> 48) as u16).max(1)
}

/// 16-bit key fingerprint (same formula as `tag_of_hash(hash(key))`).
#[inline]
fn fingerprint<K: Hash + Debug>(key: &K) -> u16 {
	tag_of_hash(hash(key))
}

/// Width of one tag-compare block in `find_tagged`.
const TAG_BLOCK: usize = 16;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// `tags[i]` is the fingerprint of `items[i]`; always `tags.len() == items.len()`.
	tags: Vec<u16>,
	/// Number of items in probation (`items[0..len0]`). In the fallback
	/// (non-SLRU) mode this always equals `items.len()`.
	len0: usize,
	/// Maximum number of items in the protected segment.
	protected_cap: usize,
	/// Total slots in this physical bucket.
	max_size: usize,
	/// Whether SLRU is active (policy is ClassicLRU).
	slru: bool,
	/// Ring of fingerprints of recently evicted probation keys (0 = empty).
	ghosts: Vec<u16>,
	/// Next ring slot to overwrite.
	ghost_head: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// A single (unpaired) bucket of `settings.bucket_size` slots.
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self::with_layout(settings, false)
	}

	/// A paired (`2 * bucket_size` slots) or unpaired (`bucket_size`) bucket.
	pub fn with_layout(settings: &AssociativeCacheSettings, paired: bool) -> Self {
		let max_size = if paired { 2 * settings.bucket_size } else { settings.bucket_size };
		// Protected gets 3/4 of the slots, keeping at least one probation slot.
		let protected_cap = (max_size * 3 / 4).min(max_size.saturating_sub(1));
		let slru = settings.eviction_policy == EvictionPolicyKind::ClassicLRU;
		let ghost_len = if slru { (max_size.saturating_add(1) / 2).max(1) } else { 0 };
		Self {
			items: Vec::with_capacity(max_size),
			tags: Vec::with_capacity(max_size),
			len0: 0,
			protected_cap,
			max_size,
			slru,
			ghosts: vec![0u16; ghost_len],
			ghost_head: 0,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	#[inline]
	fn find(&self, key: &K) -> Option<usize> {
		self.find_tagged(key, fingerprint(key))
	}

	/// Tag-first lookup. Scans `tags` forward (index 0 first) in fixed-width
	/// blocks: each block builds a match bitmask with a branch-free compare loop
	/// (auto-vectorisable), then confirms full keys only at set bits, in
	/// ascending order. A false-positive tag match continues the scan.
	#[inline]
	fn find_tagged(&self, key: &K, tag: u16) -> Option<usize> {
		debug_assert_eq!(self.tags.len(), self.items.len());
		let tags = &self.tags[..];
		let items = &self.items[..];
		let mut base = 0usize;
		let mut blocks = tags.chunks_exact(TAG_BLOCK);
		for block in &mut blocks {
			let mut mask: u32 = 0;
			for (j, &t) in block.iter().enumerate() {
				mask |= ((t == tag) as u32) << j;
			}
			while mask != 0 {
				let j = mask.trailing_zeros() as usize;
				if items[base + j].key == *key {
					return Some(base + j);
				}
				mask &= mask - 1;
			}
			base += TAG_BLOCK;
		}
		for (j, &t) in blocks.remainder().iter().enumerate() {
			if t == tag && items[base + j].key == *key {
				return Some(base + j);
			}
		}
		None
	}

	/// Recomputes every tag from its item's key (non-SLRU fallback only).
	fn rebuild_tags(&mut self) {
		self.tags.clear();
		self.tags.extend(self.items.iter().map(|it| fingerprint(&it.key)));
	}

	/// Records a tag of an evicted probation key in the ghost ring.
	#[inline]
	fn record_ghost(&mut self, tag: u16) {
		if self.ghosts.is_empty() {
			return;
		}
		self.ghosts[self.ghost_head] = tag;
		self.ghost_head += 1;
		if self.ghost_head == self.ghosts.len() {
			self.ghost_head = 0;
		}
	}

	/// Returns whether `fp` is in the ghost ring, clearing the entry if so.
	#[inline]
	fn take_ghost(&mut self, fp: u16) -> bool {
		match self.ghosts.iter().position(|&g| g == fp) {
			Some(i) => {
				self.ghosts[i] = 0;
				true
			}
			None => false,
		}
	}

	/// Evicts `items[0]` (probation LRU, or protected LRU when probation is
	/// empty), recording its tag as a ghost if it came from probation.
	#[inline]
	fn evict_front(&mut self) -> Item<K, V> {
		let from_probation = self.len0 > 0;
		let old_tag = self.tags[0];
		let old = self.remove_at(0);
		if from_probation {
			self.record_ghost(old_tag);
		}
		old
	}

	/// Applies the hit rule to the item at `pos`. Returns its new index.
	#[inline]
	fn hit(&mut self, pos: usize) -> usize {
		if !self.slru {
			let idx = self.eviction_policy.get_from_position(&mut self.items, pos);
			self.len0 = self.items.len();
			self.rebuild_tags();
			return idx;
		}
		if pos >= self.len0 {
			// Protected hit: move to protected MRU.
			self.items[pos..].rotate_left(1);
			self.tags[pos..].rotate_left(1);
			return self.items.len() - 1;
		}
		if self.protected_cap == 0 {
			// No protected segment (1-slot bucket): plain LRU in probation.
			self.items[pos..self.len0].rotate_left(1);
			self.tags[pos..self.len0].rotate_left(1);
			return self.len0 - 1;
		}
		// Probation hit: promote to protected MRU in one rotate.
		self.items[pos..].rotate_left(1);
		self.tags[pos..].rotate_left(1);
		self.len0 -= 1;
		if self.items.len() - self.len0 > self.protected_cap {
			// Demote protected LRU (now at index len0) to probation MRU.
			self.len0 += 1;
		}
		self.items.len() - 1
	}

	/// Lookup with a precomputed tag. Applies the hit rule on a hit.
	#[inline]
	pub fn pair_get_tagged(&mut self, key: &K, tag: u16) -> Option<&Item<K, V>> {
		let pos = self.find_tagged(key, tag)?;
		let j = self.hit(pos);
		Some(&self.items[j])
	}

	/// Lookup. Computes the tag from the key.
	pub fn pair_get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let tag = fingerprint(key);
		self.pair_get_tagged(key, tag)
	}

	/// Insert. Computes the tag from the key.
	pub fn pair_put(&mut self, item: Item<K, V>) -> Option<Item<K, V>> {
		let tag = fingerprint(&item.key);
		self.pair_put_tagged(item, tag)
	}

	/// Insert with a precomputed tag. Updates in place (with the hit rule) if
	/// the key is resident. Returns the item dropped from the bucket, if any.
	pub fn pair_put_tagged(&mut self, item: Item<K, V>, tag: u16) -> Option<Item<K, V>> {
		if let Some(p) = self.find_tagged(&item.key, tag) {
			self.items[p].value = item.value;
			self.hit(p);
			return None;
		}

		if !self.slru {
			let evicted = if self.items.len() >= self.max_size && !self.items.is_empty() {
				let c = self.eviction_policy.eviction_candidate_index(&self.items);
				self.tags.remove(c);
				Some(self.items.remove(c))
			} else {
				None
			};
			self.items.push(item);
			self.tags.push(tag);
			self.len0 = self.items.len();
			return evicted;
		}

		let full = self.items.len() >= self.max_size && !self.items.is_empty();
		let ghost_hit = self.take_ghost(tag);

		if ghost_hit || self.len0 == 0 {
			// Ghost hit, or all-protected fallback: v001 behaviour (insert at
			// probation MRU after evicting items[0] if full).
			let evicted = if full { Some(self.evict_front()) } else { None };
			self.items.insert(self.len0, item);
			self.tags.insert(self.len0, tag);
			self.len0 += 1;
			return evicted;
		}

		// Cold key with a non-empty probation: insert at probation middle.
		let mid = self.len0 / 2;
		if full {
			// Evict probation LRU and slide the new item to `mid` in one move.
			// Probation keeps `len0` items (len0 - 1 survivors + the new one).
			let old = std::mem::replace(&mut self.items[0], item);
			let old_tag = std::mem::replace(&mut self.tags[0], tag);
			self.items[0..=mid].rotate_left(1);
			self.tags[0..=mid].rotate_left(1);
			self.record_ghost(old_tag);
			Some(old)
		} else {
			self.items.insert(mid, item);
			self.tags.insert(mid, tag);
			self.len0 += 1;
			None
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

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		let key = item.key;
		let tag = fingerprint(&key);
		let dropped = self.pair_put_tagged(item, tag);
		// Re-find by key: the item may sit at probation MRU, probation middle,
		// or (for a resident update) protected MRU.
		let pos = self.find_tagged(&key, tag).expect("bucket invariant: just inserted");
		(&self.items[pos], dropped)
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		self.pair_get(key)
	}

	fn get_position(&self, key: &K) -> Option<usize> {
		self.find(key)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.hit(position);
		&self.items[index]
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		if index < self.len0 {
			self.len0 -= 1;
		}
		self.tags.remove(index);
		self.items.remove(index)
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for (i, item) in self.items.iter().enumerate() {
			let seg = if i < self.len0 { "prob" } else { "prot" };
			writeln!(f, "    [{seg}] {}", item)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// One paired SLRU bucket of 8 slots (protected cap 6, 4 ghost slots).
	fn bucket8() -> AssociativeCacheBucket<i64, i64> {
		bucket_with(4, true, EvictionPolicyKind::ClassicLRU)
	}

	fn bucket_with(bucket_size: usize, paired: bool, policy: EvictionPolicyKind) -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::with_layout(
			&AssociativeCacheSettings {
				num_of_buckets: 2,
				bucket_size,
				eviction_policy: policy,
			},
			paired,
		)
	}

	fn check_structure(b: &AssociativeCacheBucket<i64, i64>) {
		assert!(b.items.len() <= b.max_size);
		assert!(b.len0 <= b.items.len());
		if b.slru {
			assert!(b.items.len() - b.len0 <= b.protected_cap);
		}
		let mut seen = std::collections::HashSet::new();
		for it in &b.items {
			assert!(seen.insert(it.key), "duplicate key {}", it.key);
		}
		check_tags(b);
	}

	fn check_tags(b: &AssociativeCacheBucket<i64, i64>) {
		assert_eq!(b.tags.len(), b.items.len(), "tags/items length mismatch");
		for (i, it) in b.items.iter().enumerate() {
			assert_eq!(b.tags[i], fingerprint(&it.key), "tag mismatch at slot {i} (key {})", it.key);
		}
	}

	#[test]
	fn tag_of_hash_matches_fingerprint() {
		for k in -500i64..500 {
			assert_eq!(tag_of_hash(hash(&k)), fingerprint(&k));
			assert_ne!(fingerprint(&k), 0);
		}
	}

	#[test]
	fn tags_track_items_under_random_workload() {
		// Simple LCG; no external crates.
		let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
		let mut next = move || {
			state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			state >> 33
		};
		for (bs, paired, policy) in [
			(4usize, true, EvictionPolicyKind::ClassicLRU),
			(16, true, EvictionPolicyKind::ClassicLRU),
			(17, true, EvictionPolicyKind::ClassicLRU),
			(1, false, EvictionPolicyKind::ClassicLRU),
			(5, false, EvictionPolicyKind::ClassicLRU),
			(8, true, EvictionPolicyKind::Fifo),
			(8, true, EvictionPolicyKind::Lfu),
			(8, true, EvictionPolicyKind::Swap),
		] {
			let mut b = bucket_with(bs, paired, policy);
			let range = (b.max_size as u64) * 3 + 1;
			for _ in 0..5000 {
				let k = (next() % range) as i64;
				if b.pair_get(&k).is_none() {
					b.pair_put(Item::new(k, k));
				}
				check_structure(&b);
				assert!(b.find(&k).is_some(), "just-touched key {k} not found");
			}
		}
	}

	#[test]
	fn find_handles_tag_false_positive() {
		let mut b = bucket_with(16, true, EvictionPolicyKind::ClassicLRU);
		for k in 0..20i64 {
			b.pair_put(Item::new(k, k));
		}
		// Force every tag to collide; full-key confirm must still pick the right slot.
		let forced = 7u16;
		for t in b.tags.iter_mut() {
			*t = forced;
		}
		for (i, it) in b.items.clone().iter().enumerate() {
			assert_eq!(b.find_tagged(&it.key, forced), Some(i));
		}
		assert_eq!(b.find_tagged(&999, forced), None);
	}

	#[test]
	fn cold_key_lands_at_probation_middle() {
		let mut b = bucket8();
		for k in 1..=5i64 {
			let len0 = b.len0;
			b.pair_put(Item::new(k, k));
			assert_eq!(b.find(&k), Some(len0 / 2), "cold key {k} not at len0/2");
			check_structure(&b);
		}
		for k in 6..=8i64 {
			b.pair_put(Item::new(k, k));
		}
		assert_eq!(b.items.len(), 8);
		let len0 = b.len0;
		let evicted = b.pair_put(Item::new(100, 100));
		assert!(evicted.is_some());
		assert_eq!(b.len0, len0);
		assert_eq!(b.find(&100), Some(len0 / 2));
		check_structure(&b);
	}

	#[test]
	fn evicted_then_reput_lands_at_probation_mru() {
		let mut b = bucket8();
		for k in 1..=8i64 {
			b.pair_put(Item::new(k, k));
		}
		let victim = b.items[0].key;
		let evicted = b.pair_put(Item::new(1000, 1000)).expect("full bucket evicts");
		assert_eq!(evicted.key, victim);
		assert!(b.find(&victim).is_none());
		b.pair_put(Item::new(victim, victim));
		assert_eq!(b.find(&victim), Some(b.len0 - 1), "ghost hit must land at probation MRU");
		check_structure(&b);
	}

	#[test]
	fn long_scan_never_evicts_protected() {
		let mut b = bucket8();
		let hot = [1i64, 2, 3];
		for &k in &hot {
			b.pair_put(Item::new(k, k));
		}
		for &k in &hot {
			assert!(b.pair_get(&k).is_some());
		}
		assert_eq!(b.len0, 0);
		for k in 10_000..10_200i64 {
			assert!(b.pair_get(&k).is_none());
			b.pair_put(Item::new(k, k));
			check_structure(&b);
			for &h in &hot {
				let p = b.find(&h).expect("protected item evicted by scan");
				assert!(p >= b.len0, "hot key left protected");
			}
		}
	}

	#[test]
	fn ghost_ring_wraps_and_stays_bounded() {
		let mut b = bucket8();
		for k in 0..1000i64 {
			if b.pair_get(&(k % 37)).is_none() {
				b.pair_put(Item::new(k % 37, k));
			}
			check_structure(&b);
		}
		assert_eq!(b.ghosts.len(), 4);
		assert!(b.ghost_head < b.ghosts.len());
	}

	#[test]
	fn trait_remove_at_keeps_tags_in_sync() {
		let mut b = bucket8();
		for k in 1..=8i64 {
			b.pair_put(Item::new(k, k));
		}
		b.remove_at(3);
		check_structure(&b);
		b.evict_item();
		check_structure(&b);
	}
}
