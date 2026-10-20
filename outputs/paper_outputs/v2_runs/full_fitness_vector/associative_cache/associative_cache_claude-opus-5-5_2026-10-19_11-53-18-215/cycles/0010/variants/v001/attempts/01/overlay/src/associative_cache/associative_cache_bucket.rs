//! H03 (cycle 0004): ghost-guided insertion position on v001's SLRU.
//! H03 (cycle 0005): `Item` shrinks to 16 bytes (`{ key, value }`). The SLRU
//! path below is unchanged; it was always purely positional. The non-SLRU
//! fallback keeps per-item policy state (LFU counters) in `meta`, a `Vec<u32>`
//! parallel to `items`.
//! H01 (cycle 0006): protected cap raised from 3/4 to 7/8 of the slots, with a
//! probation floor of 1 slot (2 slots for buckets of 16 or more).
//! H01 (cycle 0007): tag-first lookup. A `tags: Vec<u16>` array parallel to
//! `items` holds the upper 16 bits of each key's hash. `find_tagged` scans the
//! tags in 16-wide blocks with a branch-free match bitmask and confirms only
//! the matching slots with a full key compare. The cache hashes each key once
//! per op and passes the tag down, so the ghost ring reuses it too. Every move
//! of `items` is mirrored on `tags`.
//! H03 (cycle 0008): cold keys (ghost miss) enter at probation LRU (index 0)
//! instead of the probation middle. When full, the newcomer simply takes the
//! evicted victim's slot (no rotate), so it is the next victim unless it is
//! hit first. Demoted-protected items and ghost-hit keys (probation MRU) thus
//! outlive one-hit wonders.
//! H01 (cycle 0009): protected cap raised to 15/16 of the slots for buckets of
//! 16 or more, with probation never narrower than 2 slots
//! (`max_size - max(max_size / 16, 2)`: 16 -> 14, 32 -> 30, 64 -> 60,
//! 128 -> 120). Smaller buckets keep the 7/8 cap with a 1-slot floor. The
//! 2-slot floor keeps probation LRU (cold entry) distinct from probation MRU
//! (demoted / ghost-hit entry).
//! H01 (cycle 0010): leaner get-hit path. A precomputed `fast_slru` flag
//! (`slru && protected_cap > 0`) is the single check on the hot path. Protected
//! and probation hits share one rotate of `items[pos..]` / `tags[pos..]`; the
//! `len0` bookkeeping (promotion + optional demotion) is branch-free.
//! Replacement decisions are identical to cycle 0009.
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
//!   - **ghost miss (cold key)**: insert at probation LRU (index 0). If full,
//!     `items[0]` (probation LRU) is evicted and replaced in place by the new
//!     item; `len0` stays unchanged.
//!   - all-protected fallback (`len0 == 0`): v001 behaviour.
//!   Every eviction of a probation item records its fingerprint in the ring.
//!
//! SLRU applies only when the configured policy is `ClassicLRU`. Any other
//! policy uses a single-segment fallback through the boxed policy, with
//! `len0 == items.len()` and `meta.len() == items.len()`.

use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use crate::shared::utils::hash;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Width of one tag-scan block (one `u16` match bit per slot).
const TAG_BLOCK: usize = 16;

/// 16-bit tag from a full key hash: the upper 16 bits, with 0 mapped to 1 so
/// that 0 marks an empty ghost slot.
#[inline]
pub(crate) fn tag_of_hash(h: usize) -> u16 {
	(((h as u64) >> 48) as u16).max(1)
}

/// 16-bit key fingerprint / tag.
#[inline]
fn fingerprint<K: Hash + Debug>(key: &K) -> u16 {
	tag_of_hash(hash(key))
}

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Tags parallel to `items`: `tags[i] == fingerprint(&items[i].key)`.
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
	/// Hot-path flag: `slru && protected_cap > 0`.
	fast_slru: bool,
	/// Ring of fingerprints of recently evicted probation keys (0 = empty).
	ghosts: Vec<u16>,
	/// Next ring slot to overwrite.
	ghost_head: usize,
	/// Fallback-only per-item policy metadata (e.g. LFU counters), parallel to
	/// `items`: `meta[i]` belongs to `items[i]`. Always empty when `slru`.
	meta: Vec<u32>,
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
		// Buckets of 16+ slots: protected gets 15/16 of the slots, but probation
		// never drops below 2 slots (16 -> 14, 32 -> 30, 64 -> 60, 128 -> 120).
		// Smaller buckets: 7/8 with a 1-slot floor (2 -> 1, 4 -> 3, 8 -> 7;
		// 1 slot -> 0).
		let protected_cap = if max_size >= 16 {
			max_size - (max_size / 16).max(2)
		} else {
			(max_size * 7 / 8).min(max_size.saturating_sub(1))
		};
		let slru = settings.eviction_policy == EvictionPolicyKind::ClassicLRU;
		let ghost_len = if slru { (max_size.saturating_add(1) / 2).max(1) } else { 0 };
		let meta = if slru { Vec::new() } else { Vec::with_capacity(max_size) };
		Self {
			items: Vec::with_capacity(max_size),
			tags: Vec::with_capacity(max_size),
			len0: 0,
			protected_cap,
			max_size,
			slru,
			fast_slru: slru && protected_cap > 0,
			ghosts: vec![0u16; ghost_len],
			ghost_head: 0,
			meta,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	#[inline]
	fn find(&self, key: &K) -> Option<usize> {
		self.find_tagged(key, fingerprint(key))
	}

	/// Tag-first lookup: per 16-slot block, build a match bitmask over the tags
	/// without branches, then confirm set bits in ascending order with a full
	/// key compare. The tail shorter than a block is scanned linearly.
	#[inline]
	fn find_tagged(&self, key: &K, tag: u16) -> Option<usize> {
		let tags = self.tags.as_slice();
		let items = self.items.as_slice();
		debug_assert_eq!(tags.len(), items.len(), "tags out of sync with items");
		let mut base = 0usize;
		let mut blocks = tags.chunks_exact(TAG_BLOCK);
		for block in &mut blocks {
			let mut mask: u32 = 0;
			for (j, &t) in block.iter().enumerate() {
				mask |= ((t == tag) as u32) << j;
			}
			while mask != 0 {
				let i = base + mask.trailing_zeros() as usize;
				if items[i].key == *key {
					return Some(i);
				}
				mask &= mask - 1;
			}
			base += TAG_BLOCK;
		}
		for (j, &t) in blocks.remainder().iter().enumerate() {
			if t == tag {
				let i = base + j;
				if items[i].key == *key {
					return Some(i);
				}
			}
		}
		None
	}

	/// Recomputes every tag from its item's key.
	#[inline(never)]
	fn rebuild_tags(&mut self) {
		self.tags.clear();
		self.tags.extend(self.items.iter().map(|it| fingerprint(&it.key)));
	}

	/// Re-establishes `tags.len() == items.len()` in case `items` was edited
	/// externally through `get_items_mut`.
	#[inline]
	fn sync_tags(&mut self) {
		if self.tags.len() != self.items.len() {
			self.rebuild_tags();
		}
	}

	/// Fallback only: re-establish `meta.len() == items.len()` in case `items`
	/// was edited externally through `get_items_mut`. New entries get 0.
	#[inline]
	fn sync_meta(&mut self) {
		if self.meta.len() != self.items.len() {
			self.meta.resize(self.items.len(), 0);
		}
	}

	/// Records the tag of an evicted probation key in the ghost ring.
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
	/// empty), recording a ghost if it came from probation.
	#[inline]
	fn evict_front(&mut self) -> Item<K, V> {
		let from_probation = self.len0 > 0;
		let tag = self.tags[0];
		let old = self.remove_at(0);
		if from_probation {
			self.record_ghost(tag);
		}
		old
	}

	/// Applies the hit rule to the item at `pos`. Returns its new index.
	#[inline(always)]
	fn hit(&mut self, pos: usize) -> usize {
		if self.fast_slru {
			// One rotate to protected MRU serves both hit kinds. Then, without
			// branches: a probation hit moves the boundary down by one
			// (promotion); if protected now exceeds its cap, the boundary moves
			// back up by one (protected LRU demoted to probation MRU). A
			// protected hit leaves `len0` as is (protected already <= cap).
			self.items[pos..].rotate_left(1);
			self.tags[pos..].rotate_left(1);
			let n = self.items.len();
			let in_prob = (pos < self.len0) as usize;
			self.len0 -= in_prob;
			self.len0 += ((n - self.len0) > self.protected_cap) as usize;
			return n - 1;
		}
		self.hit_cold(pos)
	}

	/// Cold hit path: non-SLRU fallback, or SLRU with no protected segment.
	#[inline(never)]
	fn hit_cold(&mut self, pos: usize) -> usize {
		if !self.slru {
			self.sync_meta();
			let idx = self
				.eviction_policy
				.get_from_position_meta(self.items.as_mut_slice(), self.meta.as_mut_slice(), pos);
			self.len0 = self.items.len();
			// The policy may reorder arbitrarily; re-derive the tags.
			self.rebuild_tags();
			return idx;
		}
		if pos >= self.len0 {
			// Protected hit: move to protected MRU.
			self.items[pos..].rotate_left(1);
			self.tags[pos..].rotate_left(1);
			return self.items.len() - 1;
		}
		// protected_cap == 0 (1-slot bucket): plain LRU in probation.
		self.items[pos..self.len0].rotate_left(1);
		self.tags[pos..self.len0].rotate_left(1);
		self.len0 - 1
	}

	/// Lookup with a precomputed tag (`tag_of_hash(hash(key))`).
	#[inline(always)]
	pub fn pair_get_tagged(&mut self, key: &K, tag: u16) -> Option<&Item<K, V>> {
		let pos = self.find_tagged(key, tag)?;
		let j = self.hit(pos);
		Some(&self.items[j])
	}

	/// Lookup. Scans the whole bucket once and applies the hit rule.
	pub fn pair_get(&mut self, key: &K) -> Option<&Item<K, V>> {
		self.pair_get_tagged(key, fingerprint(key))
	}

	/// Insert. Updates in place (with the hit rule) if the key is resident.
	/// Returns the item dropped from the bucket, if any.
	pub fn pair_put(&mut self, item: Item<K, V>) -> Option<Item<K, V>> {
		let tag = fingerprint(&item.key);
		self.pair_put_tagged(item, tag)
	}

	/// Insert with a precomputed tag (`tag_of_hash(hash(&item.key))`).
	pub fn pair_put_tagged(&mut self, item: Item<K, V>, tag: u16) -> Option<Item<K, V>> {
		if let Some(p) = self.find_tagged(&item.key, tag) {
			self.items[p].value = item.value;
			self.hit(p);
			return None;
		}
		self.insert_new(item, tag)
	}

	/// Inserts a key known to be non-resident.
	#[inline]
	fn insert_new(&mut self, item: Item<K, V>, tag: u16) -> Option<Item<K, V>> {
		if !self.slru {
			self.sync_meta();
			let evicted = if self.items.len() >= self.max_size && !self.items.is_empty() {
				let c = self.eviction_policy.eviction_candidate_index_meta(&self.items, &self.meta);
				self.meta.remove(c);
				self.tags.remove(c);
				Some(self.items.remove(c))
			} else {
				None
			};
			self.items.push(item);
			self.meta.push(0);
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

		// Cold key with a non-empty probation: insert at probation LRU.
		if full {
			// Evict probation LRU and take its slot in place. Probation keeps
			// `len0` items (len0 - 1 survivors + the new one at index 0).
			let old = std::mem::replace(&mut self.items[0], item);
			let old_tag = std::mem::replace(&mut self.tags[0], tag);
			self.record_ghost(old_tag);
			Some(old)
		} else {
			self.items.insert(0, item);
			self.tags.insert(0, tag);
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
		self.sync_tags();
		let key = item.key;
		let tag = fingerprint(&key);
		let dropped = self.pair_put_tagged(item, tag);
		// Re-find by key: the item may sit at probation MRU, probation LRU,
		// or (for a resident update) protected MRU.
		let pos = self.find_tagged(&key, tag).expect("bucket invariant: just inserted");
		(&self.items[pos], dropped)
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		self.sync_tags();
		self.pair_get(key)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		self.sync_tags();
		let index = self.hit(position);
		&self.items[index]
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		if !self.slru && self.meta.len() == self.items.len() {
			return Some(self.eviction_policy.eviction_candidate_index_meta(&self.items, &self.meta));
		}
		Some(self.eviction_policy.eviction_candidate_index(&self.items))
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
		if !self.slru && index < self.meta.len() {
			self.meta.remove(index);
		}
		if self.tags.len() == self.items.len() {
			self.tags.remove(index);
			self.items.remove(index)
		} else {
			let old = self.items.remove(index);
			self.rebuild_tags();
			old
		}
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
impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Cycle 0009 hit rule, verbatim. Reference for the equivalence test.
	fn hit_reference(&mut self, pos: usize) -> usize {
		if !self.slru {
			self.sync_meta();
			let idx = self
				.eviction_policy
				.get_from_position_meta(self.items.as_mut_slice(), self.meta.as_mut_slice(), pos);
			self.len0 = self.items.len();
			self.rebuild_tags();
			return idx;
		}
		if pos >= self.len0 {
			self.items[pos..].rotate_left(1);
			self.tags[pos..].rotate_left(1);
			return self.items.len() - 1;
		}
		if self.protected_cap == 0 {
			self.items[pos..self.len0].rotate_left(1);
			self.tags[pos..self.len0].rotate_left(1);
			return self.len0 - 1;
		}
		self.items[pos..].rotate_left(1);
		self.tags[pos..].rotate_left(1);
		self.len0 -= 1;
		if self.items.len() - self.len0 > self.protected_cap {
			self.len0 += 1;
		}
		self.items.len() - 1
	}

	/// `pair_get` using the cycle 0009 hit rule.
	fn ref_get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let pos = self.find(key)?;
		let j = self.hit_reference(pos);
		Some(&self.items[j])
	}

	/// `pair_put` using the cycle 0009 hit rule.
	fn ref_put(&mut self, item: Item<K, V>) -> Option<Item<K, V>> {
		let tag = fingerprint(&item.key);
		if let Some(p) = self.find_tagged(&item.key, tag) {
			self.items[p].value = item.value;
			self.hit_reference(p);
			return None;
		}
		self.insert_new(item, tag)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// One paired SLRU bucket of 8 slots (protected cap 7, 4 ghost slots).
	fn bucket8() -> AssociativeCacheBucket<i64, i64> {
		slru_bucket(4)
	}

	/// One paired SLRU bucket of `2 * bucket_size` slots.
	fn slru_bucket(bucket_size: usize) -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::with_layout(
			&AssociativeCacheSettings {
				num_of_buckets: 2,
				bucket_size,
				eviction_policy: EvictionPolicyKind::ClassicLRU,
			},
			true,
		)
	}

	/// One unpaired fallback bucket of `size` slots with `policy`.
	fn fallback(size: usize, policy: EvictionPolicyKind) -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::with_layout(
			&AssociativeCacheSettings {
				num_of_buckets: 1,
				bucket_size: size,
				eviction_policy: policy,
			},
			false,
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
		if cfg!(debug_assertions) {
			assert_eq!(b.tags.len(), b.items.len(), "tags length out of sync");
			for (i, it) in b.items.iter().enumerate() {
				assert_eq!(b.tags[i], fingerprint(&it.key), "tag mismatch at {i} (key {})", it.key);
			}
		}
	}

	#[test]
	fn fast_slru_flag() {
		assert!(bucket8().fast_slru);
		assert!(!fallback(4, EvictionPolicyKind::Lfu).fast_slru);
		// 1-slot SLRU bucket: protected_cap == 0, cold path.
		assert!(!fallback(1, EvictionPolicyKind::ClassicLRU).fast_slru);
	}

	#[test]
	fn fast_hit_matches_reference_under_10k_mixed_ops() {
		let layouts: [(usize, bool, EvictionPolicyKind); 7] = [
			(1, false, EvictionPolicyKind::ClassicLRU),
			(2, false, EvictionPolicyKind::ClassicLRU),
			(4, true, EvictionPolicyKind::ClassicLRU),
			(8, true, EvictionPolicyKind::ClassicLRU),
			(16, true, EvictionPolicyKind::ClassicLRU),
			(5, false, EvictionPolicyKind::Lfu),
			(5, false, EvictionPolicyKind::Swap),
		];
		for (bucket_size, paired, policy) in layouts {
			let settings = AssociativeCacheSettings {
				num_of_buckets: 2,
				bucket_size,
				eviction_policy: policy,
			};
			let mut new_b: AssociativeCacheBucket<i64, i64> = AssociativeCacheBucket::with_layout(&settings, paired);
			let mut ref_b: AssociativeCacheBucket<i64, i64> = AssociativeCacheBucket::with_layout(&settings, paired);
			let span = (new_b.max_size as i64) * 3 + 3;
			let mut x: u64 = 0x2545_f491_4f6c_dd1d ^ (bucket_size as u64) << 3;
			for step in 0..10_000 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				let key = if x % 3 == 0 { (x >> 8) as i64 % 4 } else { (x >> 8) as i64 % span };
				let op = (x >> 40) % 4;
				if op == 0 {
					let a = new_b.pair_put(Item::new(key, key + 1));
					let r = ref_b.ref_put(Item::new(key, key + 1));
					assert_eq!(a, r, "step {step}: put drop differs");
				} else {
					let a = new_b.pair_get(&key).cloned();
					let r = ref_b.ref_get(&key).cloned();
					assert_eq!(a, r, "step {step}: get result differs");
					if a.is_none() {
						let a = new_b.pair_put(Item::new(key, key));
						let r = ref_b.ref_put(Item::new(key, key));
						assert_eq!(a, r, "step {step}: miss-put drop differs");
					}
				}
				assert_eq!(new_b.items, ref_b.items, "step {step}: items order differs");
				assert_eq!(new_b.len0, ref_b.len0, "step {step}: len0 differs");
				assert_eq!(new_b.tags, ref_b.tags, "step {step}: tags differ");
				assert_eq!(new_b.meta, ref_b.meta, "step {step}: meta differs");
				if policy == EvictionPolicyKind::ClassicLRU {
					check_structure(&new_b);
				}
			}
		}
	}

	#[test]
	fn tag_of_hash_is_never_zero() {
		assert_eq!(tag_of_hash(0), 1);
		assert_eq!(tag_of_hash(0x0000_ffff_ffff_ffff), 1);
		assert_eq!(tag_of_hash(0xabcd_0000_0000_0000), 0xabcd);
	}

	#[test]
	fn find_tagged_matches_linear_scan_across_blocks() {
		// 32-slot bucket: two full tag blocks; 24 items: one block + remainder.
		for (bucket_size, n) in [(16usize, 32i64), (16, 24), (4, 8), (4, 3)] {
			let mut b = slru_bucket(bucket_size);
			for k in 0..n {
				b.pair_put(Item::new(k, k));
			}
			check_structure(&b);
			for k in 0..(n + 50) {
				let linear = b.items.iter().position(|it| it.key == k);
				assert_eq!(b.find(&k), linear, "key {k}");
			}
		}
	}

	#[test]
	fn protected_cap_fifteen_sixteenths_floor_two() {
		for (bucket_size, paired, expected) in [
			(1usize, false, 0usize),
			(2, false, 1),
			(2, true, 3),
			(4, true, 7),
			(8, true, 14),
			(16, true, 30),
			(32, true, 60),
			(64, true, 120),
		] {
			let b: AssociativeCacheBucket<i64, i64> = AssociativeCacheBucket::with_layout(
				&AssociativeCacheSettings {
					num_of_buckets: 2,
					bucket_size,
					eviction_policy: EvictionPolicyKind::ClassicLRU,
				},
				paired,
			);
			assert_eq!(b.protected_cap, expected, "max_size {}", b.max_size);
			if b.max_size >= 16 {
				assert!(b.max_size - b.protected_cap >= 2, "probation floor of 2 slots");
			}
		}
	}

	#[test]
	fn cold_key_lands_at_probation_lru() {
		let mut b = bucket8();
		for k in 1..=5i64 {
			b.pair_put(Item::new(k, k));
			assert_eq!(b.find(&k), Some(0), "cold key {k} not at probation LRU");
			check_structure(&b);
		}
		// Full bucket: cold key replaces the evicted probation LRU in place,
		// len0 unchanged.
		for k in 6..=8i64 {
			b.pair_put(Item::new(k, k));
		}
		assert_eq!(b.items.len(), 8);
		let len0 = b.len0;
		let victim = b.items[0].key;
		let evicted = b.pair_put(Item::new(100, 100)).expect("full bucket evicts");
		assert_eq!(evicted.key, victim);
		assert_eq!(b.len0, len0);
		assert_eq!(b.find(&100), Some(0));
		check_structure(&b);
	}

	#[test]
	fn cold_newcomer_is_next_victim() {
		// 8-slot bucket: fill probation, promote 7 keys to protected (cap 7),
		// then one more promotion demotes protected LRU to probation MRU.
		let mut b = bucket8();
		for k in 1..=8i64 {
			b.pair_put(Item::new(k, k));
		}
		for k in 1..=8i64 {
			assert!(b.pair_get(&k).is_some());
			check_structure(&b);
		}
		assert_eq!(b.len0, 1);
		let demoted = b.items[0].key;
		// Cold key evicts the demoted item (sole probation slot) and takes its
		// place; the next cold key then evicts that newcomer, not a protected one.
		let e1 = b.pair_put(Item::new(500, 500)).expect("evicts");
		assert_eq!(e1.key, demoted);
		let e2 = b.pair_put(Item::new(501, 501)).expect("evicts");
		assert_eq!(e2.key, 500);
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
		check_structure(&b);
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
		check_structure(&b);
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
	fn slru_bucket_has_no_meta() {
		let mut b = bucket8();
		for k in 0..100i64 {
			if b.pair_get(&(k % 13)).is_none() {
				b.pair_put(Item::new(k % 13, k));
			}
			check_structure(&b);
		}
		assert!(b.meta.is_empty());
	}

	#[test]
	fn tags_stay_in_lockstep_under_10k_mixed_ops() {
		for bucket_size in [4usize, 8] {
			// 8-slot and 16-slot SLRU buckets.
			let mut b = slru_bucket(bucket_size);
			let mut x: u64 = 0x9e37_79b9_7f4a_7c15 ^ bucket_size as u64;
			for _ in 0..10_000 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				// Skewed key space: a hot set plus a wide cold range.
				let key = if x % 3 == 0 { (x >> 8) as i64 % 5 } else { (x >> 8) as i64 % 61 };
				let op = (x >> 40) % 4;
				if op == 0 {
					// Plain put (resident keys take the update path).
					b.pair_put(Item::new(key, key + 1));
				} else if b.pair_get(&key).is_none() {
					b.pair_put(Item::new(key, key));
				}
				check_structure(&b);
				if op == 3 && !b.items.is_empty() && (x >> 50) % 16 == 0 {
					let idx = ((x >> 20) as usize) % b.items.len();
					let removed = b.remove_at(idx);
					assert!(b.find(&removed.key).is_none());
					check_structure(&b);
				}
			}
		}
	}

	#[test]
	fn bucket_trait_put_and_get_find_items() {
		let mut b = bucket8();
		for k in 0..20i64 {
			let (it, _) = Bucket::put(&mut b, Item::new(k, k * 10));
			assert_eq!(it.key, k);
			check_structure(&b);
			assert_eq!(Bucket::get(&mut b, &k).map(|i| i.value), Some(k * 10));
			check_structure(&b);
		}
	}

	// ==================== Fallback LFU (counters in `meta`) ====================

	#[test]
	fn lfu_get_increments_counter() {
		let mut b = fallback(4, EvictionPolicyKind::Lfu);
		b.pair_put(Item::new(1, 10));
		for expected in 1..=3u32 {
			assert!(b.pair_get(&1).is_some());
			assert_eq!(b.meta[b.find(&1).unwrap()], expected);
		}
	}

	#[test]
	fn lfu_evicts_least_frequently_used() {
		let mut b = fallback(2, EvictionPolicyKind::Lfu);
		b.pair_put(Item::new(1, 10));
		b.pair_put(Item::new(2, 20));
		b.pair_get(&1); // counter: 1
		b.pair_get(&1); // counter: 2
		b.pair_get(&2); // counter: 1
		let evicted = b.pair_put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 2);
		// The newcomer starts at 0; the survivor keeps its count.
		assert_eq!(b.meta[b.find(&1).unwrap()], 2);
		assert_eq!(b.meta[b.find(&3).unwrap()], 0);
	}

	#[test]
	fn lfu_evicts_first_on_tie() {
		let mut b = fallback(2, EvictionPolicyKind::Lfu);
		b.pair_put(Item::new(1, 10));
		b.pair_put(Item::new(2, 20));
		let evicted = b.pair_put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 1);
	}

	#[test]
	fn eviction_candidate_index_lfu() {
		let mut b = fallback(4, EvictionPolicyKind::Lfu);
		b.pair_put(Item::new(1, 10));
		b.pair_put(Item::new(2, 20));
		b.pair_get(&1);
		b.pair_get(&1);
		b.pair_get(&2);
		assert_eq!(Bucket::eviction_candidate_index(&b), Some(1));
	}

	#[test]
	fn fallback_meta_stays_parallel_under_reordering_policies() {
		for policy in [
			EvictionPolicyKind::TimestampLRU,
			EvictionPolicyKind::Swap,
			EvictionPolicyKind::Fifo,
			EvictionPolicyKind::Lifo,
			EvictionPolicyKind::Lfu,
		] {
			let mut b = fallback(5, policy);
			for k in 0..500i64 {
				let key = (k * 7919) % 23;
				if b.pair_get(&key).is_none() {
					b.pair_put(Item::new(key, key));
				}
				assert_eq!(b.meta.len(), b.items.len(), "{policy:?}: meta out of sync");
				assert_eq!(b.len0, b.items.len());
				assert!(b.items.len() <= b.max_size);
				check_structure(&b);
			}
			let removed = b.remove_at(0);
			assert!(b.find(&removed.key).is_none());
			assert_eq!(b.meta.len(), b.items.len());
			check_structure(&b);
		}
	}

	#[test]
	fn timestamp_lru_fallback_evicts_least_recently_touched() {
		let mut b = fallback(3, EvictionPolicyKind::TimestampLRU);
		b.pair_put(Item::new(1, 10));
		b.pair_put(Item::new(2, 20));
		b.pair_put(Item::new(3, 30));
		b.pair_get(&1);
		let evicted = b.pair_put(Item::new(4, 40));
		assert_eq!(evicted.unwrap().key, 2);
	}
}
