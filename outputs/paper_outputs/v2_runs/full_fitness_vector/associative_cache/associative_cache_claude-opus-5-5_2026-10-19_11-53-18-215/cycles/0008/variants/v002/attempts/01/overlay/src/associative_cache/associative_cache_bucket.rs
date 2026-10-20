//! H03 (cycle 0004): ghost-guided insertion position on v001's SLRU.
//! H03 (cycle 0005): `Item` shrinks to 16 bytes (`{ key, value }`). The SLRU
//! path below is unchanged; it was always purely positional. The non-SLRU
//! fallback keeps per-item policy state (LFU counters) in `meta`, a `Vec<u32>`
//! parallel to `items`.
//! H01 (cycle 0006): protected cap raised from 3/4 to 7/8 of the slots, with a
//! probation floor of 1 slot (2 slots for buckets of 16 or more).
//! H01 (cycle 0007): tag-first lookup. A tag array parallel to `items` holds
//! the upper 16 bits of each key's hash. `find_tagged` scans the tags in
//! 16-wide blocks with a branch-free match bitmask and confirms only the
//! matching slots with a full key compare. The cache hashes each key once per
//! op and passes the tag down, so the ghost ring reuses it too. Every move of
//! `items` is mirrored on the tags.
//! H02 (cycle 0008): hot/cold bucket layout. No replacement decision changes.
//! - Tags live inline in the bucket struct (`[u16; INLINE_TAGS]`, 32 slots).
//!   Only buckets with `max_size > INLINE_TAGS` use a heap `Box<[u16]>`.
//!   The live tag count is `tags_len`, equal to `items.len()` when in sync.
//! - `len0` and `protected_cap` are `u32`. `max_size` and the `slru` flag
//!   share one `u32` word (bit 31 = slru).
//! - `#[repr(C)]` puts `items`, `len0`, `protected_cap`, the size/flag word and
//!   the first tags in the first 64 bytes.
//! - The ghost ring, `ghost_head`, `meta` and the boxed eviction policy move to
//!   a `Box<ColdState>`. The SLRU get path (`pair_get_tagged` / `hit`) never
//!   dereferences it.
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

/// Number of tags stored inline in the bucket struct.
const INLINE_TAGS: usize = 32;

/// Bit of `size_flags` marking an SLRU bucket. The low 31 bits hold `max_size`.
const SLRU_BIT: u32 = 1 << 31;

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

/// State only the miss/put path and the non-SLRU fallback touch.
#[derive(Debug)]
struct ColdState<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Ring of fingerprints of recently evicted probation keys (0 = empty).
	ghosts: Vec<u16>,
	/// Next ring slot to overwrite.
	ghost_head: usize,
	/// Fallback-only per-item policy metadata (e.g. LFU counters), parallel to
	/// `items`: `meta[i]` belongs to `items[i]`. Always empty when `slru`.
	meta: Vec<u32>,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

#[derive(Debug)]
#[repr(C)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Number of items in probation (`items[0..len0]`). In the fallback
	/// (non-SLRU) mode this always equals `items.len()`.
	len0: u32,
	/// Maximum number of items in the protected segment.
	protected_cap: u32,
	/// `max_size` (low 31 bits) | `SLRU_BIT` when SLRU is active.
	size_flags: u32,
	/// Inline tags: `tags[i] == fingerprint(&items[i].key)` for `i < tags_len`.
	/// Unused when `tags_heap` is `Some`.
	tags: [u16; INLINE_TAGS],
	/// Number of live tags; equals `items.len()` unless `items` was edited
	/// externally through `get_items_mut`.
	tags_len: u32,
	/// Tag storage for buckets with more than `INLINE_TAGS` slots.
	tags_heap: Option<Box<[u16]>>,
	cold: Box<ColdState<K, V>>,
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
		assert!(max_size < SLRU_BIT as usize, "bucket max_size {max_size} too large");
		// Protected gets 7/8 of the slots, keeping a probation floor scaled to
		// bucket size: 4 -> 3, 8 -> 7, 16 -> 14, 32 -> 28; 1 slot -> 0.
		let probation_min = if max_size >= 16 { 2 } else { 1 };
		let protected_cap = (max_size * 7 / 8).min(max_size.saturating_sub(probation_min));
		let slru = settings.eviction_policy == EvictionPolicyKind::ClassicLRU;
		let ghost_len = if slru { (max_size.saturating_add(1) / 2).max(1) } else { 0 };
		let meta = if slru { Vec::new() } else { Vec::with_capacity(max_size) };
		let tags_heap = if max_size > INLINE_TAGS {
			Some(vec![0u16; max_size].into_boxed_slice())
		} else {
			None
		};
		Self {
			items: Vec::with_capacity(max_size),
			len0: 0,
			protected_cap: protected_cap as u32,
			size_flags: max_size as u32 | if slru { SLRU_BIT } else { 0 },
			tags: [0u16; INLINE_TAGS],
			tags_len: 0,
			tags_heap,
			cold: Box::new(ColdState {
				ghosts: vec![0u16; ghost_len],
				ghost_head: 0,
				meta,
				eviction_policy: settings.eviction_policy.build(),
			}),
		}
	}

	#[inline]
	fn max_size(&self) -> usize {
		(self.size_flags & !SLRU_BIT) as usize
	}

	#[inline]
	fn slru(&self) -> bool {
		self.size_flags & SLRU_BIT != 0
	}

	/// Full tag buffer (inline or heap), including unused slots.
	#[inline]
	fn tag_buf(&self) -> &[u16] {
		match &self.tags_heap {
			Some(h) => &h[..],
			None => &self.tags[..],
		}
	}

	#[inline]
	fn tag_buf_mut(&mut self) -> &mut [u16] {
		match &mut self.tags_heap {
			Some(h) => &mut h[..],
			None => &mut self.tags[..],
		}
	}

	/// Grows the tag buffer (moving to the heap) so it holds at least `n` tags.
	/// Only reachable when `items` outgrows `max_size` through external edits.
	#[cold]
	#[inline(never)]
	fn grow_tags(&mut self, n: usize) {
		let cap = n.max(self.max_size()).max(INLINE_TAGS);
		let live = self.tags_len as usize;
		let mut v = vec![0u16; cap].into_boxed_slice();
		v[..live].copy_from_slice(&self.tag_buf()[..live]);
		self.tags_heap = Some(v);
	}

	#[inline]
	fn ensure_tag_cap(&mut self, n: usize) {
		if n > self.tag_buf().len() {
			self.grow_tags(n);
		}
	}

	/// Inserts `tag` at `idx`, shifting `tags[idx..tags_len]` right by one.
	#[inline]
	fn tag_insert(&mut self, idx: usize, tag: u16) {
		let n = self.tags_len as usize;
		self.ensure_tag_cap(n + 1);
		let buf = self.tag_buf_mut();
		buf.copy_within(idx..n, idx + 1);
		buf[idx] = tag;
		self.tags_len += 1;
	}

	/// Removes the tag at `idx`, shifting `tags[idx + 1..tags_len]` left by one.
	#[inline]
	fn tag_remove(&mut self, idx: usize) -> u16 {
		let n = self.tags_len as usize;
		let buf = self.tag_buf_mut();
		let t = buf[idx];
		buf.copy_within(idx + 1..n, idx);
		self.tags_len -= 1;
		t
	}

	#[inline]
	fn tag_push(&mut self, tag: u16) {
		let n = self.tags_len as usize;
		self.tag_insert(n, tag);
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
		let items = self.items.as_slice();
		let buf = self.tag_buf();
		debug_assert_eq!(self.tags_len as usize, items.len(), "tags out of sync with items");
		let n = items.len().min(buf.len());
		let tags = &buf[..n];
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
		let n = self.items.len();
		self.ensure_tag_cap(n);
		let buf: &mut [u16] = match &mut self.tags_heap {
			Some(h) => &mut h[..],
			None => &mut self.tags[..],
		};
		for (d, it) in buf.iter_mut().zip(self.items.iter()) {
			*d = fingerprint(&it.key);
		}
		self.tags_len = n as u32;
	}

	/// Re-establishes `tags_len == items.len()` in case `items` was edited
	/// externally through `get_items_mut`.
	#[inline]
	fn sync_tags(&mut self) {
		if self.tags_len as usize != self.items.len() {
			self.rebuild_tags();
		}
	}

	/// Fallback only: re-establish `meta.len() == items.len()` in case `items`
	/// was edited externally through `get_items_mut`. New entries get 0.
	#[inline]
	fn sync_meta(&mut self) {
		let n = self.items.len();
		if self.cold.meta.len() != n {
			self.cold.meta.resize(n, 0);
		}
	}

	/// Records the tag of an evicted probation key in the ghost ring.
	#[inline]
	fn record_ghost(&mut self, tag: u16) {
		let cold = &mut *self.cold;
		if cold.ghosts.is_empty() {
			return;
		}
		cold.ghosts[cold.ghost_head] = tag;
		cold.ghost_head += 1;
		if cold.ghost_head == cold.ghosts.len() {
			cold.ghost_head = 0;
		}
	}

	/// Returns whether `fp` is in the ghost ring, clearing the entry if so.
	#[inline]
	fn take_ghost(&mut self, fp: u16) -> bool {
		let ghosts = &mut self.cold.ghosts;
		match ghosts.iter().position(|&g| g == fp) {
			Some(i) => {
				ghosts[i] = 0;
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
		let tag = self.tag_buf()[0];
		let old = self.remove_at(0);
		if from_probation {
			self.record_ghost(tag);
		}
		old
	}

	/// Non-SLRU hit: dispatch through the boxed policy (cold state).
	#[inline(never)]
	fn hit_fallback(&mut self, pos: usize) -> usize {
		self.sync_meta();
		let cold = &mut *self.cold;
		let idx = cold
			.eviction_policy
			.get_from_position_meta(self.items.as_mut_slice(), cold.meta.as_mut_slice(), pos);
		self.len0 = self.items.len() as u32;
		// The policy may reorder arbitrarily; re-derive the tags.
		self.rebuild_tags();
		idx
	}

	/// Applies the hit rule to the item at `pos`. Returns its new index.
	#[inline]
	fn hit(&mut self, pos: usize) -> usize {
		if !self.slru() {
			return self.hit_fallback(pos);
		}
		let len = self.items.len();
		let len0 = self.len0 as usize;
		if pos >= len0 {
			// Protected hit: move to protected MRU.
			self.items[pos..].rotate_left(1);
			self.tag_buf_mut()[pos..len].rotate_left(1);
			return len - 1;
		}
		if self.protected_cap == 0 {
			// No protected segment (1-slot bucket): plain LRU in probation.
			self.items[pos..len0].rotate_left(1);
			self.tag_buf_mut()[pos..len0].rotate_left(1);
			return len0 - 1;
		}
		// Probation hit: promote to protected MRU in one rotate.
		self.items[pos..].rotate_left(1);
		self.tag_buf_mut()[pos..len].rotate_left(1);
		let mut new_len0 = len0 - 1;
		if len - new_len0 > self.protected_cap as usize {
			// Demote protected LRU (now at index len0) to probation MRU.
			new_len0 += 1;
		}
		self.len0 = new_len0 as u32;
		len - 1
	}

	/// Lookup with a precomputed tag (`tag_of_hash(hash(key))`).
	#[inline]
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

		let max_size = self.max_size();

		if !self.slru() {
			self.sync_meta();
			let evicted = if self.items.len() >= max_size && !self.items.is_empty() {
				let cold = &mut *self.cold;
				let c = cold.eviction_policy.eviction_candidate_index_meta(&self.items, &cold.meta);
				cold.meta.remove(c);
				self.tag_remove(c);
				Some(self.items.remove(c))
			} else {
				None
			};
			self.items.push(item);
			self.cold.meta.push(0);
			self.tag_push(tag);
			self.len0 = self.items.len() as u32;
			return evicted;
		}

		let full = self.items.len() >= max_size && !self.items.is_empty();
		let ghost_hit = self.take_ghost(tag);

		if ghost_hit || self.len0 == 0 {
			// Ghost hit, or all-protected fallback: v001 behaviour (insert at
			// probation MRU after evicting items[0] if full).
			let evicted = if full { Some(self.evict_front()) } else { None };
			let len0 = self.len0 as usize;
			self.items.insert(len0, item);
			self.tag_insert(len0, tag);
			self.len0 += 1;
			return evicted;
		}

		// Cold key with a non-empty probation: insert at probation middle.
		let mid = self.len0 as usize / 2;
		if full {
			// Evict probation LRU and slide the new item to `mid` in one move.
			// Probation keeps `len0` items (len0 - 1 survivors + the new one).
			let old = std::mem::replace(&mut self.items[0], item);
			self.items[0..=mid].rotate_left(1);
			let buf = self.tag_buf_mut();
			let old_tag = std::mem::replace(&mut buf[0], tag);
			buf[0..=mid].rotate_left(1);
			self.record_ghost(old_tag);
			Some(old)
		} else {
			self.items.insert(mid, item);
			self.tag_insert(mid, tag);
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
		self.max_size()
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.cold.eviction_policy
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		self.sync_tags();
		let key = item.key;
		let tag = fingerprint(&key);
		let dropped = self.pair_put_tagged(item, tag);
		// Re-find by key: the item may sit at probation MRU, probation middle,
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
		if !self.slru() && self.cold.meta.len() == self.items.len() {
			return Some(self.cold.eviction_policy.eviction_candidate_index_meta(&self.items, &self.cold.meta));
		}
		Some(self.cold.eviction_policy.eviction_candidate_index(&self.items))
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		if index < self.len0 as usize {
			self.len0 -= 1;
		}
		if !self.slru() && index < self.cold.meta.len() {
			self.cold.meta.remove(index);
		}
		if self.tags_len as usize == self.items.len() {
			self.tag_remove(index);
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
		let len0 = self.len0 as usize;
		for (i, item) in self.items.iter().enumerate() {
			let seg = if i < len0 { "prob" } else { "prot" };
			writeln!(f, "    [{seg}] {}", item)?;
		}
		Ok(())
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

	fn len0(b: &AssociativeCacheBucket<i64, i64>) -> usize {
		b.len0 as usize
	}

	fn check_structure(b: &AssociativeCacheBucket<i64, i64>) {
		assert!(b.items.len() <= b.max_size());
		assert!(len0(b) <= b.items.len());
		assert!(b.items.len() - len0(b) <= b.protected_cap as usize);
		let mut seen = std::collections::HashSet::new();
		for it in &b.items {
			assert!(seen.insert(it.key), "duplicate key {}", it.key);
		}
		assert_eq!(b.tags_len as usize, b.items.len(), "tags length out of sync");
		let tags = &b.tag_buf()[..b.items.len()];
		for (i, it) in b.items.iter().enumerate() {
			assert_eq!(tags[i], fingerprint(&it.key), "tag mismatch at {i} (key {})", it.key);
		}
	}

	#[test]
	fn tag_of_hash_is_never_zero() {
		assert_eq!(tag_of_hash(0), 1);
		assert_eq!(tag_of_hash(0x0000_ffff_ffff_ffff), 1);
		assert_eq!(tag_of_hash(0xabcd_0000_0000_0000), 0xabcd);
	}

	#[test]
	fn small_buckets_keep_tags_inline() {
		for bucket_size in [1usize, 2, 4, 8, 16] {
			let b = slru_bucket(bucket_size);
			assert!(b.tags_heap.is_none(), "max_size {} must not allocate tags", b.max_size());
		}
		let big = slru_bucket(17);
		assert_eq!(big.tags_heap.as_ref().map(|h| h.len()), Some(34));
	}

	#[test]
	fn large_bucket_uses_heap_tags() {
		// 64-slot bucket: tags on the heap; behaviour identical.
		let mut b = slru_bucket(32);
		for k in 0..500i64 {
			let key = (k * 7919) % 101;
			if b.pair_get(&key).is_none() {
				b.pair_put(Item::new(key, key));
			}
			check_structure(&b);
		}
		for k in 0..150i64 {
			let linear = b.items.iter().position(|it| it.key == k);
			assert_eq!(b.find(&k), linear, "key {k}");
		}
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
	fn protected_cap_is_seven_eighths_with_scaled_floor() {
		for (bucket_size, paired, expected) in [
			(1usize, false, 0usize),
			(2, false, 1),
			(2, true, 3),
			(4, true, 7),
			(8, true, 14),
			(16, true, 28),
		] {
			let b: AssociativeCacheBucket<i64, i64> = AssociativeCacheBucket::with_layout(
				&AssociativeCacheSettings {
					num_of_buckets: 2,
					bucket_size,
					eviction_policy: EvictionPolicyKind::ClassicLRU,
				},
				paired,
			);
			assert_eq!(b.protected_cap as usize, expected, "max_size {}", b.max_size());
		}
	}

	#[test]
	fn cold_key_lands_at_probation_middle() {
		let mut b = bucket8();
		for k in 1..=5i64 {
			let l0 = len0(&b);
			b.pair_put(Item::new(k, k));
			assert_eq!(b.find(&k), Some(l0 / 2), "cold key {k} not at len0/2");
			check_structure(&b);
		}
		// Full bucket: cold key still lands at the middle, len0 unchanged.
		for k in 6..=8i64 {
			b.pair_put(Item::new(k, k));
		}
		assert_eq!(b.items.len(), 8);
		let l0 = len0(&b);
		let evicted = b.pair_put(Item::new(100, 100));
		assert!(evicted.is_some());
		assert_eq!(len0(&b), l0);
		assert_eq!(b.find(&100), Some(l0 / 2));
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
		assert_eq!(b.find(&victim), Some(len0(&b) - 1), "ghost hit must land at probation MRU");
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
				assert!(p >= len0(&b), "hot key left protected");
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
		assert_eq!(b.cold.ghosts.len(), 4);
		assert!(b.cold.ghost_head < b.cold.ghosts.len());
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
		assert!(b.cold.meta.is_empty());
	}

	#[test]
	fn tags_stay_in_lockstep_under_10k_mixed_ops() {
		for bucket_size in [4usize, 8, 16, 24] {
			// 8-, 16-, 32-slot (inline) and 48-slot (heap) SLRU buckets.
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
	fn external_edits_resync_tags() {
		let mut b = bucket8();
		for k in 0..6i64 {
			b.pair_put(Item::new(k, k));
		}
		// External edit: drop one item and push past the slot count.
		b.get_items_mut().remove(2);
		assert!(Bucket::get(&mut b, &100).is_none());
		assert_eq!(b.tags_len as usize, b.items.len());
		for k in 200..240i64 {
			b.get_items_mut().push(Item::new(k, k));
		}
		assert_eq!(Bucket::get(&mut b, &239).map(|i| i.value), Some(239));
		assert_eq!(b.tags_len as usize, b.items.len());
		let tags = &b.tag_buf()[..b.items.len()];
		for (i, it) in b.items.iter().enumerate() {
			assert_eq!(tags[i], fingerprint(&it.key));
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
			assert_eq!(b.cold.meta[b.find(&1).unwrap()], expected);
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
		assert_eq!(b.cold.meta[b.find(&1).unwrap()], 2);
		assert_eq!(b.cold.meta[b.find(&3).unwrap()], 0);
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
				assert_eq!(b.cold.meta.len(), b.items.len(), "{policy:?}: meta out of sync");
				assert_eq!(len0(&b), b.items.len());
				assert!(b.items.len() <= b.max_size());
				check_structure(&b);
			}
			let removed = b.remove_at(0);
			assert!(b.find(&removed.key).is_none());
			assert_eq!(b.cold.meta.len(), b.items.len());
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
