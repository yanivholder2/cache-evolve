//! H03 (cycle 0004): ghost-guided insertion position on v001's 24-byte SLRU.
//!
//! One physical bucket still merges a pair of logical buckets
//! (`2 * bucket_size` slots when paired, `bucket_size` for the trailing
//! unpaired bucket), backed by one contiguous `Vec` split at `len0`:
//!
//! - `items[0..len0]` is **probation** (index 0 = LRU, `len0 - 1` = MRU).
//! - `items[len0..]` is **protected** (index `len0` = LRU, last = MRU).
//!
//! - get, protected hit: rotate to protected MRU.
//! - get, probation hit: promote to protected MRU. If protected then exceeds
//!   `protected_cap`, its LRU item is demoted to probation's MRU end by moving
//!   the boundary (`len0 += 1`). A get never drops an item.
//! - put (new key): a per-bucket ring of 16-bit fingerprints remembers
//!   recently evicted probation keys.
//!   - **ghost hit**: clear the ghost entry, evict `items[0]` if full, insert
//!     at probation MRU (index `len0`). Never inserted into protected.
//!   - **ghost miss (cold key)**: insert at probation middle `len0 / 2`. If
//!     full, `items[0]` (probation LRU) is evicted and the new item is placed
//!     at the middle with one short rotate; `len0` stays unchanged.
//!   - all-protected fallback (`len0 == 0`): v001 behaviour.
//!   Every eviction of a probation item records its fingerprint in the ring.
//!
//! SLRU applies only when the configured policy is `ClassicLRU`. Any other
//! policy uses a single-segment fallback through the boxed policy, with
//! `len0 == items.len()`.

use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use crate::shared::utils::hash;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// 16-bit key fingerprint: upper 16 bits of the key hash, with 0 mapped to 1
/// so that 0 marks an empty ghost slot.
#[inline]
fn fingerprint<K: Hash + Debug>(key: &K) -> u16 {
	let h = hash(key) as u64;
	let fp = (h >> 48) as u16;
	if fp == 0 {
		1
	} else {
		fp
	}
}

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
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
		self.items.iter().position(|d| d.key == *key)
	}

	/// Records the fingerprint of an evicted probation key in the ghost ring.
	#[inline]
	fn record_ghost(&mut self, key: &K) {
		if self.ghosts.is_empty() {
			return;
		}
		self.ghosts[self.ghost_head] = fingerprint(key);
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
	/// empty), recording a ghost if it came from probation.
	#[inline]
	fn evict_front(&mut self) -> Item<K, V> {
		let from_probation = self.len0 > 0;
		let old = self.remove_at(0);
		if from_probation {
			self.record_ghost(&old.key);
		}
		old
	}

	/// Applies the hit rule to the item at `pos`. Returns its new index.
	#[inline]
	fn hit(&mut self, pos: usize) -> usize {
		if !self.slru {
			let idx = self.eviction_policy.get_from_position(&mut self.items, pos);
			self.len0 = self.items.len();
			return idx;
		}
		if pos >= self.len0 {
			// Protected hit: move to protected MRU.
			self.items[pos..].rotate_left(1);
			return self.items.len() - 1;
		}
		if self.protected_cap == 0 {
			// No protected segment (1-slot bucket): plain LRU in probation.
			self.items[pos..self.len0].rotate_left(1);
			return self.len0 - 1;
		}
		// Probation hit: promote to protected MRU in one rotate.
		self.items[pos..].rotate_left(1);
		self.len0 -= 1;
		if self.items.len() - self.len0 > self.protected_cap {
			// Demote protected LRU (now at index len0) to probation MRU.
			self.len0 += 1;
		}
		self.items.len() - 1
	}

	/// Lookup. Scans the whole bucket once and applies the hit rule.
	pub fn pair_get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let pos = self.find(key)?;
		let j = self.hit(pos);
		Some(&self.items[j])
	}

	/// Insert. Updates in place (with the hit rule) if the key is resident.
	/// Returns the item dropped from the bucket, if any.
	pub fn pair_put(&mut self, item: Item<K, V>) -> Option<Item<K, V>> {
		if let Some(p) = self.find(&item.key) {
			self.items[p].value = item.value;
			self.hit(p);
			return None;
		}

		if !self.slru {
			let evicted = if self.items.len() >= self.max_size && !self.items.is_empty() {
				let c = self.eviction_policy.eviction_candidate_index(&self.items);
				Some(self.items.remove(c))
			} else {
				None
			};
			self.items.push(item);
			self.len0 = self.items.len();
			return evicted;
		}

		let full = self.items.len() >= self.max_size && !self.items.is_empty();
		let ghost_hit = self.take_ghost(fingerprint(&item.key));

		if ghost_hit || self.len0 == 0 {
			// Ghost hit, or all-protected fallback: v001 behaviour (insert at
			// probation MRU after evicting items[0] if full).
			let evicted = if full { Some(self.evict_front()) } else { None };
			self.items.insert(self.len0, item);
			self.len0 += 1;
			return evicted;
		}

		// Cold key with a non-empty probation: insert at probation middle.
		let mid = self.len0 / 2;
		if full {
			// Evict probation LRU and slide the new item to `mid` in one move.
			// Probation keeps `len0` items (len0 - 1 survivors + the new one).
			let old = std::mem::replace(&mut self.items[0], item);
			self.items[0..=mid].rotate_left(1);
			self.record_ghost(&old.key);
			Some(old)
		} else {
			self.items.insert(mid, item);
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
		let dropped = self.pair_put(item);
		// Re-find by key: the item may sit at probation MRU, probation middle,
		// or (for a resident update) protected MRU.
		let pos = self.find(&key).expect("bucket invariant: just inserted");
		(&self.items[pos], dropped)
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		self.pair_get(key)
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
		AssociativeCacheBucket::with_layout(
			&AssociativeCacheSettings {
				num_of_buckets: 2,
				bucket_size: 4,
				eviction_policy: EvictionPolicyKind::ClassicLRU,
			},
			true,
		)
	}

	fn check_structure(b: &AssociativeCacheBucket<i64, i64>) {
		assert!(b.items.len() <= b.max_size);
		assert!(b.len0 <= b.items.len());
		assert!(b.items.len() - b.len0 <= b.protected_cap);
		let mut seen = std::collections::HashSet::new();
		for it in &b.items {
			assert!(seen.insert(it.key), "duplicate key {}", it.key);
		}
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
		// Full bucket: cold key still lands at the middle, len0 unchanged.
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
}
