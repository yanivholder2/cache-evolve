use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
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
	/// Dense per-item key fingerprints, maintained only when `lip` is true.
	/// Invariant (lip): `tags.len() == items.len()` and
	/// `tags[i] == tag_of(&items[i].key)` after every public operation.
	/// Mutating `items` through `get_items_mut()` bypasses this mirror; no
	/// in-crate caller does so.
	tags: Vec<u8>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// True when the policy is ClassicLRU (decided once at construction).
	lip: bool,
	/// True when the policy's eviction victim is always index 0 (checked once
	/// at construction), enabling the fused evict+insert rotation on full
	/// buckets with no per-miss dynamic call. Only ever true for ClassicLRU,
	/// so it also gates the inline move-to-back hit path.
	victim_is_front: bool,
	/// Per-bucket miss counter driving the bimodal MRU insertion.
	bip_tick: u32,
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
		let victim_is_front = lip && eviction_policy.eviction_candidate_index(&[]) == 0;
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			tags: if lip { Vec::with_capacity(settings.bucket_size) } else { Vec::new() },
			max_size: settings.bucket_size,
			eviction_policy,
			lip,
			victim_is_front,
			bip_tick: 0,
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
		if self.lip && self.victim_is_front {
			// Inline ClassicLRU move-to-back. ClassicLRU's `get_from_position`
			// is `remove(position); push(item)` and writes no Item field, so
			// one in-place rotation of the suffix is order-equivalent. `len`
			// is unchanged; an MRU hit (position == n-1) moves nothing.
			let n = self.items.len();
			debug_assert!(position < n, "get_from_position: {position} out of bounds (len={n})");
			debug_assert_eq!(self.tags.len(), n);
			if position + 1 < n {
				self.items[position..].rotate_left(1);
				self.tags[position..].rotate_left(1);
			}
			self.debug_check_tags();
			return &self.items[n - 1];
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		if self.lip {
			// ClassicLRU moved `position` to the back; mirror it in `tags`.
			debug_assert_eq!(index, self.items.len() - 1);
			let t = self.tags.remove(position);
			self.tags.push(t);
		}
		self.debug_check_tags();
		&self.items[index]
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}
		let idx = self.eviction_policy.eviction_candidate_index(&self.items);
		let evicted = self.items.remove(idx);
		if self.lip {
			self.tags.remove(idx);
		}
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
		if self.lip {
			self.tags.remove(index);
		}
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

			self.bip_tick = self.bip_tick.wrapping_add(1);
			let bip = self.bip_tick % BIP_MRU_PERIOD == 0;

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
		self.items.push(item);
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

	fn bucket(size: usize) -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::new(&AssociativeCacheSettings {
			num_of_buckets: 1,
			bucket_size: size,
			eviction_policy: EvictionPolicyKind::ClassicLRU,
		})
	}

	fn check(b: &AssociativeCacheBucket<i64, i64>) {
		assert_eq!(b.tags.len(), b.items.len());
		for (t, it) in b.tags.iter().zip(&b.items) {
			assert_eq!(*t, tag_of(&it.key));
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

	#[test]
	fn hit_at_mru_end_is_noop_and_hit_elsewhere_moves_to_back() {
		let mut b = bucket(8);
		for k in 0..8i64 {
			b.put(Item::new(k, k * 10));
		}
		check(&b);
		let before: Vec<i64> = b.items.iter().map(|i| i.key).collect();
		let mru = *before.last().unwrap();
		let got = b.get(&mru).expect("MRU key resident");
		assert_eq!(got.key, mru);
		assert_eq!(got.value, mru * 10);
		let after: Vec<i64> = b.items.iter().map(|i| i.key).collect();
		assert_eq!(after, before, "MRU hit must not reorder");
		check(&b);

		// Hit the LRU front: it must land at the back, everything else shifts.
		let front = before[0];
		let got = b.get(&front).expect("front key resident");
		assert_eq!(got.key, front);
		let mut expected = before[1..].to_vec();
		expected.push(front);
		let after: Vec<i64> = b.items.iter().map(|i| i.key).collect();
		assert_eq!(after, expected);
		check(&b);

		// Single-item bucket: the only item is MRU.
		let mut one = bucket(1);
		one.put(Item::new(5, 50));
		assert_eq!(one.get(&5).unwrap().value, 50);
		assert_eq!(one.items.len(), 1);
		check(&one);
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
