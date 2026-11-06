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

/// Bimodal insertion (BIP), width-keyed: buckets with
/// `max_size <= BIP_WIDE_BUCKET` admit every 32nd new-key put at the MRU end
/// (mask `BIP_MASK_NARROW`); wider buckets admit every 64th
/// (mask `BIP_MASK_WIDE`). All other new-key puts go to quarter depth.
const BIP_WIDE_BUCKET: usize = 256;
const BIP_MASK_NARROW: u32 = 31;
const BIP_MASK_WIDE: u32 = 63;

/// BIP MRU-admission mask for a bucket of `max_size` slots. Derived from the
/// already-loaded `max_size`, so the bucket gains no field.
#[inline]
fn bip_mask_for(max_size: usize) -> u32 {
	if max_size <= BIP_WIDE_BUCKET {
		BIP_MASK_NARROW
	} else {
		BIP_MASK_WIDE
	}
}

/// Width of one tag-scan chunk (one `u16` match mask).
const TAG_CHUNK: usize = 16;

/// Minimal multiplicative hasher used to derive both the bucket index and the
/// 1-byte key fingerprint. Deliberately not SipHash: it runs on every get/put.
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

/// The one per-op key hash: full `TagHasher` value. The cache takes the bucket
/// index from bits 24..=55 (`utils::bucket_index_from_hash`) and the tag from
/// bits 56..=63 (`tag_from_hash`), so the two never overlap.
#[inline]
pub(crate) fn tag_hash<K: Hash>(key: &K) -> u64 {
	let mut h = TagHasher(0);
	key.hash(&mut h);
	h.finish()
}

/// 1-byte fingerprint from a full `tag_hash` value: its top byte.
#[inline]
pub(crate) fn tag_from_hash(h: u64) -> u8 {
	(h >> 56) as u8
}

/// 1-byte fingerprint of a key (trait-path callers that do not pre-hash).
#[inline]
fn tag_of<K: Hash>(key: &K) -> u8 {
	tag_from_hash(tag_hash(key))
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
	/// On this path the hit move-to-back is done inline in the bucket, not via
	/// the boxed policy (H02: no virtual call on the hit path).
	lip: bool,
	/// True when the policy's eviction victim is always index 0 (checked once
	/// at construction), enabling the fused evict+insert rotation on full
	/// buckets and a dispatch-free `evict_item`.
	victim_is_front: bool,
	/// Shared per-bucket counter (the bucket's only counter field):
	/// - lip (ClassicLRU): the BIP miss counter driving bimodal MRU insertion.
	///   ClassicLRU never reads `timestamp`, so no item is stamped.
	/// - other policies: the logical clock that stamps `Item::timestamp` on
	///   insertion and on every hit (replaces the global atomic clock).
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
		let victim_is_front = lip && eviction_policy.eviction_candidate_index(&[]) == 0;
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			tags: if lip { Vec::with_capacity(settings.bucket_size) } else { Vec::new() },
			max_size: settings.bucket_size,
			eviction_policy,
			lip,
			victim_is_front,
			clock: 0,
		}
	}

	/// True when this bucket runs the tagged ClassicLRU (LIP/BIP) path.
	#[inline]
	pub fn is_lip(&self) -> bool {
		self.lip
	}

	/// Lookup with a pre-computed tag `t == tag_from_hash(tag_hash(key))`.
	/// The lip branch of `Bucket::get` without re-hashing. Non-lip buckets
	/// fall back to the trait `get` (which ignores `t`).
	#[inline]
	pub fn get_hashed(&mut self, key: &K, t: u8) -> Option<&Item<K, V>> {
		if !self.lip {
			return Bucket::get(self, key);
		}
		debug_assert_eq!(t, tag_of(key));
		let position = self.get_position_tagged(key, t)?;
		Some(self.get_from_position(position))
	}

	/// Insert with a pre-computed tag `t == tag_from_hash(tag_hash(&item.key))`.
	/// The lip branch of `Bucket::put` without re-hashing. Non-lip buckets
	/// fall back to the trait `put` (which ignores `t`).
	#[inline]
	pub fn put_hashed(&mut self, item: Item<K, V>, t: u8) -> (&Item<K, V>, Option<Item<K, V>>) {
		if !self.lip {
			return Bucket::put(self, item);
		}
		debug_assert_eq!(t, tag_of(&item.key));
		self.put_lip(item, t)
	}

	/// Advances the per-bucket clock and returns the new stamp.
	#[inline]
	fn next_stamp(&mut self) -> u32 {
		self.clock = self.clock.wrapping_add(1);
		self.clock
	}

	/// ClassicLRU hit, inlined: move `position` to the back of both `items` and
	/// `tags`. Same order `ClassicLRU::get_from_position` produces, without the
	/// vtable call. A hit on the back slot is a no-op.
	#[inline]
	fn lip_promote(&mut self, position: usize) -> &Item<K, V> {
		debug_assert!(self.lip);
		let len = self.items.len();
		if position == len - 1 {
			return &self.items[position];
		}
		let it = self.items.remove(position);
		self.items.push(it);
		let t = self.tags.remove(position);
		self.tags.push(t);
		let index = self.items.len() - 1;
		debug_assert_eq!(index, len - 1);
		self.debug_check_tags();
		&self.items[index]
	}

	/// ClassicLRU (LIP/BIP) put with the key's tag supplied by the caller.
	#[inline]
	fn put_lip(&mut self, item: Item<K, V>, t: u8) -> (&Item<K, V>, Option<Item<K, V>>) {
		debug_assert!(self.lip);
		if let Some(position) = self.get_position_tagged(&item.key, t) {
			self.items[position].value = item.value;
			let pushed_item = self.lip_promote(position);
			return (pushed_item, None);
		}

		// `clock` is the BIP miss counter here; no timestamp is written.
		// Increment-then-test, as in the parent; the mask is 31 (== % 32)
		// on narrow buckets and 63 (1/64 MRU admission) on wide ones.
		self.clock = self.clock.wrapping_add(1);
		let bip_mask = bip_mask_for(self.max_size);
		let bip = (self.clock & bip_mask) == 0;

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
		(&self.items[insert_pos], evicted_item)
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

	#[inline]
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.lip {
			// ClassicLRU: inline move-to-back (no dynamic dispatch).
			return self.lip_promote(position);
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		// The parent's timestamp-reading policy (TimestampLRU) stamped the
		// hit from the global clock; re-stamp from the per-bucket clock so
		// every stamp in this bucket comes from one monotone source.
		let stamp = self.next_stamp();
		self.items[index].timestamp = stamp;
		self.debug_check_tags();
		&self.items[index]
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}
		// lip with a front victim (ClassicLRU): the index is known, skip the
		// vtable call.
		let idx = if self.victim_is_front {
			0
		} else {
			self.eviction_policy.eviction_candidate_index(&self.items)
		};
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

	fn put(&mut self, mut item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if self.lip {
			// Trait callers do not pre-hash: hash once here and reuse the tag
			// for lookup and insertion.
			let t = tag_of(&item.key);
			return self.put_lip(item, t);
		}

		// All other policies: default behaviour (append at the back).
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.clock = self.clock.wrapping_add(1);
		item.timestamp = self.clock;
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

	/// Parent layout: Vec (24) + Vec (24) + usize (8) + Box<dyn> (16) +
	/// 2 x bool (2) + u32 (4) = 78 bytes, padded to 8-byte alignment = 80.
	/// The width-keyed BIP mask is derived from `max_size`, not stored, so
	/// the bucket stride must not change.
	#[cfg(target_pointer_width = "64")]
	#[test]
	fn bucket_size_matches_parent() {
		assert_eq!(std::mem::size_of::<AssociativeCacheBucket<u64, u64>>(), 80);
	}

	#[test]
	fn tag_of_is_top_byte_of_tag_hash() {
		for k in [-5i64, 0, 1, 2, 4096, 1 << 40, i64::MAX, i64::MIN] {
			let h = tag_hash(&k);
			assert_eq!(tag_of(&k), (h >> 56) as u8);
			assert_eq!(tag_from_hash(h), tag_of(&k));
		}
	}

	/// The hashed entry points are behaviourally identical to the trait ones.
	#[test]
	fn hashed_paths_match_trait_paths() {
		for kind in EvictionPolicyKind::all() {
			for &size in &[1usize, 3, 17, 300] {
				let mut a = bucket_with(size, kind);
				let mut b = bucket_with(size, kind);
				let mut s: u64 = 0xDEAD_BEEF ^ size as u64;
				let space = (size as u64) * 3 + 2;
				for _ in 0..5_000 {
					s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
					let key = ((s >> 33) % space) as i64;
					let t = tag_from_hash(tag_hash(&key));
					let ha = a.get_hashed(&key, t).map(|i| i.value);
					let hb = b.get(&key).map(|i| i.value);
					assert_eq!(ha, hb);
					if ha.is_none() {
						let (ia, ea) = a.put_hashed(Item::new(key, key * 3), t);
						let (ia, ea) = (ia.key, ea.map(|i| i.key));
						let (ib, eb) = b.put(Item::new(key, key * 3));
						assert_eq!(ia, ib.key);
						assert_eq!(ea, eb.map(|i| i.key));
					}
					let ka: Vec<i64> = a.items.iter().map(|i| i.key).collect();
					let kb: Vec<i64> = b.items.iter().map(|i| i.key).collect();
					assert_eq!(ka, kb, "kind={kind:?} size={size}");
					if a.lip {
						check(&a);
					}
				}
			}
		}
	}

	#[test]
	fn bip_mask_threshold() {
		assert_eq!(bip_mask_for(1), 31);
		assert_eq!(bip_mask_for(128), 31);
		assert_eq!(bip_mask_for(BIP_WIDE_BUCKET), 31);
		assert_eq!(bip_mask_for(BIP_WIDE_BUCKET + 1), 63);
		assert_eq!(bip_mask_for(1024), 63);
		// Narrow mask is bit-identical to the parent's `% 32`.
		for c in 0..10_000u32 {
			assert_eq!((c & 31) == 0, c % 32 == 0);
		}
	}

	/// Every miss is a distinct new key; the k-th miss (1-based) must land at
	/// the MRU end exactly when k is a multiple of the width-keyed period.
	#[test]
	fn bip_period_is_width_keyed() {
		for &(size, period) in &[(256usize, 32u32), (300, 64), (1024, 64)] {
			let mut b = bucket(size);
			let mut mru_count = 0u32;
			let misses = 20 * size as u32;
			for k in 1..=misses {
				let key = k as i64;
				assert!(b.get(&key).is_none());
				let len_before = b.items.len();
				let (ins, _) = b.put(Item::new(key, key));
				assert_eq!(ins.key, key);
				let at_mru = b.items.last().map(|i| i.key) == Some(key);
				// Quarter-depth insertion is distinguishable from MRU only when
				// it does not coincide with the back slot.
				let lip_pos = if len_before == size { (size - 1) / INSERT_DEPTH_DIVISOR } else { len_before / INSERT_DEPTH_DIVISOR };
				let back = if len_before == size { size - 1 } else { len_before };
				if lip_pos != back {
					assert_eq!(at_mru, k % period == 0, "size={size} miss={k}");
				}
				if k % period == 0 {
					assert!(at_mru, "size={size} miss={k}");
					mru_count += 1;
				}
				assert!(b.size() <= size);
			}
			assert_eq!(mru_count, misses / period, "size={size}");
			check(&b);
		}
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
			let stamps: Vec<u32> = b.items.iter().map(|i| i.timestamp).collect();
			assert_eq!(
				stamps,
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
			assert!(b.tags.is_empty());
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

	/// Reference model of the parent's remove-then-insert sequence, with the
	/// width-keyed BIP period (32 for max <= 256, else 64).
	struct RefBucket {
		keys: Vec<i64>,
		max: usize,
		tick: u32,
	}

	impl RefBucket {
		fn period(&self) -> u32 {
			if self.max <= 256 {
				32
			} else {
				64
			}
		}

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
			if self.tick % self.period() == 0 {
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
		for &size in &[1usize, 2, 3, 4, 5, 8, 16, 17, 37, 64, 256, 300, 1024] {
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

	/// H02: the inlined ClassicLRU hit (with its back-slot fast path) yields
	/// the same order as the reference remove/push, under traffic where most
	/// hits land on the last (MRU) slot. Also exercises the hashed entry
	/// points and in-place value update on the back slot.
	#[test]
	fn inline_hit_matches_reference_last_slot_heavy() {
		for &size in &[1usize, 2, 17, 300, 1024] {
			let mut b = bucket(size);
			let mut r = RefBucket { keys: Vec::new(), max: size, tick: 0 };
			let mut s: u64 = 0x0BAD_F00D_CAFE_1234 ^ size as u64;
			let space = (size as u64) * 3 + 2;
			let mut back_hits = 0u32;
			for step in 0..20_000u32 {
				s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
				let roll = (s >> 60) as u32; // 0..16
				let key = if roll < 10 && !r.keys.is_empty() {
					// ~5/8 of ops hit the current MRU slot.
					*r.keys.last().unwrap()
				} else if roll < 12 && r.keys.len() >= 2 {
					// Some hits just below the back slot.
					r.keys[r.keys.len() - 2]
				} else {
					((s >> 33) % space) as i64
				};
				let was_back = r.keys.last() == Some(&key);
				let t = tag_from_hash(tag_hash(&key));
				let hit_b = if step % 2 == 0 {
					b.get_hashed(&key, t).map(|i| i.key)
				} else {
					b.get(&key).map(|i| i.key)
				};
				let hit_r = r.get(key);
				assert_eq!(hit_b.is_some(), hit_r, "size={size} step={step}");
				if let Some(k) = hit_b {
					assert_eq!(k, key);
					if was_back {
						back_hits += 1;
					}
				} else {
					let (ins, ev) = b.put_hashed(Item::new(key, key * 2), t);
					assert_eq!(ins.key, key);
					let ev_r = r.put(key);
					assert_eq!(ev.map(|i| i.key), ev_r);
				}
				let order: Vec<i64> = b.items.iter().map(|i| i.key).collect();
				assert_eq!(order, r.keys, "order diverged at size {size} step {step}");
				check(&b);
				assert!(b.size() <= size);
			}
			assert!(back_hits > 1_000, "size={size}: too few back-slot hits ({back_hits})");
			// Value update on an existing key (back slot and elsewhere) keeps
			// order identical to a reference hit.
			let keys = r.keys.clone();
			for &k in keys.iter().rev().take(3) {
				let (ins, ev) = b.put(Item::new(k, k * 7));
				assert_eq!(ins.key, k);
				assert_eq!(ins.value, k * 7);
				assert!(ev.is_none());
				assert!(r.get(k));
				let order: Vec<i64> = b.items.iter().map(|i| i.key).collect();
				assert_eq!(order, r.keys, "update diverged at size {size}");
				check(&b);
			}
		}
	}
}
