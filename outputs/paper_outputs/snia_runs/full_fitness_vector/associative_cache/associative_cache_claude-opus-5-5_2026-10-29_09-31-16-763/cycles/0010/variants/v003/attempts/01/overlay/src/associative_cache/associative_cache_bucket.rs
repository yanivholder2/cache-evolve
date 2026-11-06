use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::{meta_set, Item, META_STRIDE};
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::{EvictionPolicy, MetaUse};
use log::{debug, log_enabled, Level};

use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// ClassicLRU insertion depth: a newly admitted item is inserted at
/// `len / INSERT_DEPTH_DIVISOR` (measured from the eviction end, index 0)
/// instead of at the MRU end. A hit still promotes it to the back.
const INSERT_DEPTH_DIVISOR: usize = 4;

/// Bimodal insertion (BIP): every `BIP_MRU_PERIOD`-th new-key put in a bucket
/// is inserted at the MRU end instead of at quarter depth.
const BIP_MRU_PERIOD: u32 = 32;

/// Width of one tag-scan chunk (one `u16` match mask).
const TAG_CHUNK: usize = 16;

/// Minimal multiplicative hasher used only to derive 1-byte key fingerprints.
/// Deliberately not SipHash: the tag is computed on every get/put.
struct TagHasher(u64);

const TAG_MUL: u64 = 0x9E37_79B9_7F4A_7C15;

impl Hasher for TagHasher {
	#[inline]
	fn write_u64(&mut self, x: u64) {
		self.0 = (self.0 ^ x).wrapping_mul(TAG_MUL);
	}

	#[inline]
	fn write_i64(&mut self, x: i64) {
		self.write_u64(x as u64);
	}

	#[inline]
	fn write_u32(&mut self, x: u32) {
		self.write_u64(x as u64);
	}

	#[inline]
	fn write_i32(&mut self, x: i32) {
		self.write_u64(x as u64);
	}

	#[inline]
	fn write_usize(&mut self, x: usize) {
		self.write_u64(x as u64);
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for c in &mut chunks {
			let mut buf = [0u8; 8];
			buf.copy_from_slice(c);
			self.write_u64(u64::from_le_bytes(buf));
		}
		let rem = chunks.remainder();
		if !rem.is_empty() {
			let mut buf = [0u8; 8];
			buf[..rem.len()].copy_from_slice(rem);
			self.write_u64(u64::from_le_bytes(buf));
		}
	}

	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}
}

/// 1-byte fingerprint of a key: top byte of a cheap multiplicative hash, so it
/// is independent of the low bits used for bucket selection.
#[inline]
fn tag_of<K: Hash>(key: &K) -> u8 {
	let mut h = TagHasher(0);
	key.hash(&mut h);
	(h.finish() >> 56) as u8
}

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Per-item side byte buffer, in lockstep with `items`. Its role depends on
	/// the policy (fixed at construction), so the bucket gains no field:
	/// - lip (ClassicLRU): one key fingerprint byte per item.
	///   Invariant: `tags.len() == items.len()` and
	///   `tags[i] == tag_of(&items[i].key)` after every public operation.
	/// - other policies: `META_STRIDE` bytes per item, one little-endian `u32`
	///   of policy metadata (TimestampLRU stamp / LFU counter), read and
	///   written via `meta_get`/`meta_set`.
	///   Invariant: `tags.len() == items.len() * META_STRIDE`.
	///
	/// Mutating `items` through `get_items_mut()` bypasses this mirror; no
	/// in-crate caller does so.
	tags: Vec<u8>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// True when the policy is ClassicLRU (decided once at construction).
	lip: bool,
	/// True when the policy's eviction victim is always index 0 (checked once
	/// at construction), enabling the fused evict+insert rotation on full
	/// buckets with no per-miss dynamic call.
	victim_is_front: bool,
	/// Shared per-bucket counter (the bucket's only counter field):
	/// - lip (ClassicLRU): the BIP miss counter driving bimodal MRU insertion.
	///   ClassicLRU keeps no per-item metadata, so nothing is stamped.
	/// - stamp policies (TimestampLRU): the logical clock that stamps the
	///   item's metadata slot on insertion and on every hit.
	clock: u32,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let eviction_policy: Box<dyn EvictionPolicy<K, V>> = settings.eviction_policy.build();
		let lip = eviction_policy.name() == "ClassicLRU";
		// ClassicLRU ignores `items` and always returns 0; only queried when lip
		// (other policies may require a non-empty slice).
		let victim_is_front = lip && eviction_policy.eviction_candidate_index(&[], &[]) == 0;
		let stride = if lip { 1 } else { META_STRIDE };
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			tags: Vec::with_capacity(settings.bucket_size * stride),
			max_size: settings.bucket_size,
			eviction_policy,
			lip,
			victim_is_front,
			clock: 0,
		}
	}

	/// Advances the per-bucket clock and returns the new stamp.
	#[inline]
	fn next_stamp(&mut self) -> u32 {
		self.clock = self.clock.wrapping_add(1);
		self.clock
	}

	/// Policy metadata view of the side buffer: empty on lip buckets (where
	/// the buffer holds tags, not metadata).
	#[inline]
	fn meta(&self) -> &[u8] {
		if self.lip {
			let empty: &[u8] = &[];
			empty
		} else {
			self.tags.as_slice()
		}
	}

	/// Removes the side-buffer slot of item `index` (1-byte stride on lip,
	/// `META_STRIDE` otherwise). Call right after `items.remove(index)`.
	#[inline]
	fn buf_remove(&mut self, index: usize) {
		if self.lip {
			self.tags.remove(index);
		} else {
			let o = index * META_STRIDE;
			self.tags.drain(o..o + META_STRIDE);
		}
	}

	/// Tag-filtered lookup (lip path only). Scans `tags` in 16-wide chunks,
	/// building a branch-free match mask, and full-compares only candidates.
	#[inline]
	fn get_position_tagged(&self, key: &K, t: u8) -> Option<usize> {
		debug_assert_eq!(self.tags.len(), self.items.len());
		let items = &self.items[..self.tags.len()];
		let mut base = 0usize;
		let mut chunks = self.tags.chunks_exact(TAG_CHUNK);
		for c in &mut chunks {
			let mut m: u16 = 0;
			for (j, &b) in c.iter().enumerate() {
				m |= ((b == t) as u16) << j;
			}
			while m != 0 {
				let j = m.trailing_zeros() as usize;
				if items[base + j].key == *key {
					return Some(base + j);
				}
				m &= m - 1;
			}
			base += TAG_CHUNK;
		}
		for (j, &b) in chunks.remainder().iter().enumerate() {
			if b == t && items[base + j].key == *key {
				return Some(base + j);
			}
		}
		None
	}

	/// Fused evict-front + insert-at-`p` on a full bucket. Equivalent to
	/// `items.remove(0); items.insert(p, item)` (and the same on `tags`), but
	/// moves only `items[1..=p]` down by one slot; `len` is unchanged, so no
	/// tail memmove and no reallocation. Returns the evicted front item.
	#[inline]
	fn fused_evict_front_insert(&mut self, item: Item<K, V>, t: u8, p: usize) -> Item<K, V> {
		let n = self.items.len();
		assert!(p < n, "fused_evict_front_insert: p={p} out of bounds (len={n})");
		debug_assert_eq!(self.tags.len(), n);
		let base = self.items.as_mut_ptr();
		// SAFETY: `p < n == len`, so indices 0..=p are initialised slots of the
		// buffer. We move `items[0]` out by `ptr::read`, then memmove
		// `items[1..=p]` into `items[0..p]` (overlap handled by `ptr::copy`),
		// leaving slot `p` logically uninitialised (its value now lives at
		// `p-1`, or was moved out if `p == 0`), which `ptr::write` fills without
		// dropping. Nothing between the read and the write can panic, so every
		// slot is initialised exactly once when control leaves this block.
		let evicted = unsafe {
			let evicted = std::ptr::read(base);
			std::ptr::copy(base.add(1), base, p);
			std::ptr::write(base.add(p), item);
			evicted
		};
		self.tags.copy_within(1..=p, 0);
		self.tags[p] = t;
		if log_enabled!(Level::Debug) {
			debug!(
				"Bucket.evict_item: policy={} evicted_key={:?}",
				self.eviction_policy.name(),
				evicted.key
			);
		}
		self.debug_check_tags();
		evicted
	}

	#[inline]
	fn debug_check_tags(&self) {
		if self.lip {
			debug_assert_eq!(self.tags.len(), self.items.len(), "tags out of lockstep with items");
		} else {
			debug_assert_eq!(
				self.tags.len(),
				self.items.len() * META_STRIDE,
				"meta out of lockstep with items"
			);
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

	fn get_meta(&self) -> &[u8] {
		self.meta()
	}

	/// ClassicLRU: fingerprint-filtered scan. Keys are unique within a bucket,
	/// so the forward tag scan returns the same index as the parent's reverse
	/// scan. Other policies keep the plain linear scan.
	#[inline]
	fn get_position(&self, key: &K) -> Option<usize> {
		if self.lip {
			self.get_position_tagged(key, tag_of(key))
		} else {
			self.items.iter().position(|it| it.key == *key)
		}
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = if self.lip {
			// ClassicLRU keeps no metadata: hand it an empty buffer and mirror
			// its move-to-back in `tags` here.
			let index = self.eviction_policy.get_from_position(&mut self.items, &mut [], position);
			debug_assert_eq!(index, self.items.len() - 1);
			let t = self.tags.remove(position);
			self.tags.push(t);
			index
		} else {
			let index = self
				.eviction_policy
				.get_from_position(&mut self.items, &mut self.tags, position);
			if self.eviction_policy.meta_use() == MetaUse::Stamp {
				// The policy (TimestampLRU) stamped the hit from the global
				// clock; re-stamp from the per-bucket clock so every stamp in
				// this bucket comes from one monotone source.
				let stamp = self.next_stamp();
				meta_set(&mut self.tags, index, stamp);
			}
			index
		};
		self.debug_check_tags();
		&self.items[index]
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}
		let idx = self.eviction_policy.eviction_candidate_index(&self.items, self.meta());
		let evicted = self.items.remove(idx);
		self.buf_remove(idx);
		debug!(
			"Bucket.evict_item: policy={} evicted_key={:?}",
			self.eviction_policy.name(),
			evicted.key
		);
		self.debug_check_tags();
		Some(evicted)
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		let removed = self.items.remove(index);
		self.buf_remove(index);
		self.debug_check_tags();
		removed
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if self.lip {
			// Hash once per op; reuse the tag for lookup and insertion.
			let t = tag_of(&item.key);
			if let Some(position) = self.get_position_tagged(&item.key, t) {
				self.items[position].value = item.value;
				let pushed_item = self.get_from_position(position);
				return (pushed_item, None);
			}

			// `clock` is the BIP miss counter here; no metadata is written.
			self.clock = self.clock.wrapping_add(1);
			let bip = self.clock % BIP_MRU_PERIOD == 0;

			let n = self.items.len();
			if self.victim_is_front && n > 0 && self.is_full() {
				// Fused path: the parent's remove(0) followed by insert at
				// `(n-1)/4` (LIP) or push (BIP) yields the same order as moving
				// `items[1..=p]` down one slot and writing the new item at `p`.
				let p = if bip { n - 1 } else { (n - 1) / INSERT_DEPTH_DIVISOR };
				let evicted = self.fused_evict_front_insert(item, t, p);
				return (&self.items[p], Some(evicted));
			}

			let evicted_item = if self.is_full() { self.evict_item() } else { None };

			if bip {
				// Bimodal MRU insertion: occasionally admit at the MRU end so a
				// changing working set can enter the protected region.
				self.items.push(item);
				self.tags.push(t);
				self.debug_check_tags();
				let pushed_item = self.items.last().expect("bucket invariant: just pushed");
				return (pushed_item, evicted_item);
			}
			// LIP-style insertion: place the new item a quarter of the way from
			// the LRU end (index 0) so one-hit scan items are evicted early.
			let insert_pos = self.items.len() / INSERT_DEPTH_DIVISOR;
			self.items.insert(insert_pos, item);
			self.tags.insert(insert_pos, t);
			self.debug_check_tags();
			return (&self.items[insert_pos], evicted_item);
		}

		// All other policies: default behaviour (append at the back).
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		// New slot: stamp policies get the next bucket-clock tick, counters and
		// stateless policies start at 0.
		let init = match self.eviction_policy.meta_use() {
			MetaUse::Stamp => self.next_stamp(),
			MetaUse::Counter | MetaUse::None => 0,
		};
		self.items.push(item);
		self.tags.extend_from_slice(&init.to_le_bytes());
		self.debug_check_tags();
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
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

#[cfg(test)]
mod tag_tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;
	use crate::shared::item::meta_get;

	fn bucket(size: usize) -> AssociativeCacheBucket<i64, i64> {
		bucket_with(size, EvictionPolicyKind::ClassicLRU)
	}

	fn bucket_with(size: usize, kind: EvictionPolicyKind) -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::new(&AssociativeCacheSettings {
			num_of_buckets: 1,
			bucket_size: size,
			eviction_policy: kind,
		})
	}

	fn check(b: &AssociativeCacheBucket<i64, i64>) {
		assert_eq!(b.tags.len(), b.items.len());
		for (t, it) in b.tags.iter().zip(&b.items) {
			assert_eq!(*t, tag_of(&it.key));
		}
	}

	/// Test helper: per-item metadata words of a non-lip bucket, read from the
	/// side buffer.
	fn meta_words(b: &AssociativeCacheBucket<i64, i64>) -> Vec<u32> {
		assert!(!b.lip);
		assert_eq!(b.tags.len(), b.items.len() * META_STRIDE);
		(0..b.items.len()).map(|i| meta_get(&b.tags, i)).collect()
	}

	/// Parent layout: Vec (24) + Vec (24) + usize (8) + Box<dyn> (16) +
	/// 2 x bool (2) + u32 (4) = 78 bytes, padded to 8-byte alignment = 80.
	/// Moving per-item metadata into the existing `tags` buffer must not
	/// change the bucket stride.
	#[cfg(target_pointer_width = "64")]
	#[test]
	fn bucket_size_matches_parent() {
		assert_eq!(std::mem::size_of::<AssociativeCacheBucket<u64, u64>>(), 80);
	}

	#[test]
	fn timestamp_lru_uses_bucket_clock() {
		for &start in &[0u32, u32::MAX - 2] {
			let mut b = bucket_with(3, EvictionPolicyKind::TimestampLRU);
			b.clock = start;
			b.put(Item::new(1, 10));
			b.put(Item::new(2, 20));
			b.put(Item::new(3, 30));
			assert!(b.get(&1).is_some());
			assert!(b.get(&3).is_some());
			// Stamps come from the bucket clock: insertion + hits = 5 ticks.
			assert_eq!(b.clock, start.wrapping_add(5));
			assert_eq!(
				meta_words(&b),
				vec![start.wrapping_add(4), start.wrapping_add(2), start.wrapping_add(5)],
				"start={start}"
			);
			// 2 is the oldest-touched item.
			let (_, ev) = b.put(Item::new(4, 40));
			assert_eq!(ev.map(|i| i.key), Some(2), "start={start}");
			assert!(b.get(&2).is_none());
			// Now 1 (hit at tick 4) is older than 3 (tick 5) and 4 (tick 6).
			let (_, ev) = b.put(Item::new(5, 50));
			assert_eq!(ev.map(|i| i.key), Some(1), "start={start}");
			let keys: Vec<i64> = b.items.iter().map(|i| i.key).collect();
			assert_eq!(keys, vec![3, 4, 5]);
			assert_eq!(meta_words(&b), vec![start.wrapping_add(5), start.wrapping_add(6), start.wrapping_add(7)]);
		}
	}

	#[test]
	fn lfu_counts_in_side_buffer() {
		let mut b = bucket_with(2, EvictionPolicyKind::Lfu);
		b.put(Item::new(1, 10));
		b.put(Item::new(2, 20));
		assert_eq!(meta_words(&b), vec![0, 0]);
		b.get(&1);
		b.get(&1);
		b.get(&2);
		assert_eq!(meta_words(&b), vec![2, 1]);
		let (_, ev) = b.put(Item::new(3, 30));
		assert_eq!(ev.map(|i| i.key), Some(2));
		assert_eq!(meta_words(&b), vec![2, 0]);
		// Counters are not clobbered by the clock.
		assert_eq!(b.clock, 0);
	}

	#[test]
	fn meta_stays_in_lockstep_for_non_lip_policies() {
		for kind in EvictionPolicyKind::all() {
			if kind == EvictionPolicyKind::ClassicLRU {
				continue;
			}
			let mut b = bucket_with(5, kind);
			for k in 0..2000i64 {
				let key = (k * 7919) % 13;
				if b.get(&key).is_none() {
					b.put(Item::new(key, key));
				}
				assert_eq!(b.tags.len(), b.items.len() * META_STRIDE, "{kind:?}");
				assert!(b.size() <= 5);
			}
		}
	}

	#[test]
	fn victim_is_front_for_classic_lru_only() {
		assert!(bucket(4).victim_is_front);
		for kind in EvictionPolicyKind::all() {
			let b: AssociativeCacheBucket<i64, i64> = AssociativeCacheBucket::new(&AssociativeCacheSettings {
				num_of_buckets: 1,
				bucket_size: 4,
				eviction_policy: kind,
			});
			assert_eq!(b.victim_is_front, kind == EvictionPolicyKind::ClassicLRU);
		}
	}

	#[test]
	fn tags_stay_in_lockstep() {
		let mut b = bucket(37);
		for k in 0..2000i64 {
			let key = (k * 7919) % 101;
			if b.get(&key).is_none() {
				b.put(Item::new(key, key));
			}
			check(&b);
			assert!(b.size() <= 37);
		}
		for it in b.items.clone() {
			assert_eq!(b.get_position(&it.key), b.items.iter().position(|x| x.key == it.key));
		}
		assert!(b.get(&10_000).is_none());
	}

	/// Reference model of the parent's remove-then-insert sequence.
	struct RefBucket {
		keys: Vec<i64>,
		max: usize,
		tick: u32,
	}

	impl RefBucket {
		fn get(&mut self, key: i64) -> bool {
			if let Some(p) = self.keys.iter().position(|&k| k == key) {
				let k = self.keys.remove(p);
				self.keys.push(k);
				true
			} else {
				false
			}
		}

		fn put(&mut self, key: i64) -> Option<i64> {
			let evicted = if self.keys.len() == self.max { Some(self.keys.remove(0)) } else { None };
			self.tick = self.tick.wrapping_add(1);
			if self.tick % BIP_MRU_PERIOD == 0 {
				self.keys.push(key);
			} else {
				let p = self.keys.len() / INSERT_DEPTH_DIVISOR;
				self.keys.insert(p, key);
			}
			evicted
		}
	}

	#[test]
	fn fused_matches_reference_remove_insert() {
		for &size in &[1usize, 2, 3, 4, 5, 8, 16, 17, 37, 64] {
			let mut b = bucket(size);
			let mut r = RefBucket { keys: Vec::new(), max: size, tick: 0 };
			let mut s: u64 = 0x1234_5678_9ABC_DEF0 ^ size as u64;
			let space = (size as u64) * 3 + 2;
			for _ in 0..20_000 {
				s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
				let key = ((s >> 33) % space) as i64;
				let hit_b = b.get(&key).is_some();
				let hit_r = r.get(key);
				assert_eq!(hit_b, hit_r);
				if !hit_b {
					let (ins, ev) = b.put(Item::new(key, key * 2));
					assert_eq!(ins.key, key);
					assert_eq!(ins.value, key * 2);
					let ev_r = r.put(key);
					assert_eq!(ev.map(|i| i.key), ev_r);
				}
				let order: Vec<i64> = b.items.iter().map(|i| i.key).collect();
				assert_eq!(order, r.keys, "order diverged at size {size}");
				check(&b);
				assert!(b.size() <= size);
			}
		}
	}
}
