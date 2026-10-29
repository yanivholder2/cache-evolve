use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::{debug, log_enabled, Level};

use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

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
///
/// It also records whether the key was hashed with exactly one integer write
/// (and no byte-slice write). In that case `h = x * TAG_MUL` with an odd
/// multiplier, which is a bijection on u64, so `h` identifies the key exactly
/// (assuming, as for all primitive integers, `Hash` writes the whole key).
struct TagHasher {
	h: u64,
	writes: u8,
	bytes_used: bool,
}

const TAG_MUL: u64 = 0x9E37_79B9_7F4A_7C15;

impl Hasher for TagHasher {
	#[inline]
	fn write_u64(&mut self, x: u64) {
		self.h = (self.h ^ x).wrapping_mul(TAG_MUL);
		self.writes = self.writes.wrapping_add(1);
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
		self.bytes_used = true;
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
		self.h
	}
}

/// Full tag hash of a key plus whether that hash identifies the key exactly
/// (single integer write, no byte writes; see [`TagHasher`]).
#[inline]
fn tag_hash<K: Hash>(key: &K) -> (u64, bool) {
	let mut h = TagHasher { h: 0, writes: 0, bytes_used: false };
	key.hash(&mut h);
	(h.finish(), h.writes == 1 && !h.bytes_used)
}

/// 1-byte fingerprint of a key: top byte of a cheap multiplicative hash, so it
/// is independent of the low bits used for bucket selection.
#[inline]
fn tag_of<K: Hash>(key: &K) -> u8 {
	(tag_hash(key).0 >> 56) as u8
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
	/// Miss memo (lip only): exact tag hash of the last key a `get_position`
	/// scan confirmed absent. Written from `&self`, hence an atomic, but only
	/// ever with plain Relaxed load/store (no RMW, no LOCK prefix).
	/// Invariant: if `miss_valid`, the key whose exact hash is `miss_hash` is
	/// not in `items`.
	miss_hash: AtomicU64,
	/// True when the miss memo is valid. Cleared by every mutating path.
	miss_valid: AtomicBool,
	/// True when the policy is ClassicLRU (decided once at construction).
	lip: bool,
	/// True when the policy's eviction victim is always index 0 (checked once
	/// at construction), enabling the fused evict+insert rotation on full
	/// buckets with no per-miss dynamic call.
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
			miss_hash: AtomicU64::new(0),
			miss_valid: AtomicBool::new(false),
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

	/// Invalidates the miss memo (exclusive access: plain store, no atomic op).
	#[inline]
	fn clear_memo(&mut self) {
		*self.miss_valid.get_mut() = false;
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
		self.clear_memo();
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
		// The caller may change membership behind our back: drop the memo.
		self.clear_memo();
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
	/// scan. A confirmed miss on an exactly-hashed key is memoised so the
	/// following `put` of that key can skip its own scan. Other policies keep
	/// the plain linear scan.
	#[inline]
	fn get_position(&self, key: &K) -> Option<usize> {
		if self.lip {
			let (h, exact) = tag_hash(key);
			let pos = self.get_position_tagged(key, (h >> 56) as u8);
			if pos.is_none() && exact {
				self.miss_hash.store(h, Ordering::Relaxed);
				self.miss_valid.store(true, Ordering::Relaxed);
			}
			pos
		} else {
			self.items.iter().position(|it| it.key == *key)
		}
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		if self.lip {
			// ClassicLRU moved `position` to the back; mirror it in `tags`.
			debug_assert_eq!(index, self.items.len() - 1);
			let t = self.tags.remove(position);
			self.tags.push(t);
		} else {
			// The parent's timestamp-reading policy (TimestampLRU) stamped the
			// hit from the global clock; re-stamp from the per-bucket clock so
			// every stamp in this bucket comes from one monotone source.
			let stamp = self.next_stamp();
			self.items[index].timestamp = stamp;
		}
		self.debug_check_tags();
		&self.items[index]
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		self.clear_memo();
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
		self.clear_memo();
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
			// Hash once per op; reuse the hash for the memo check, the lookup
			// and insertion.
			let (h, exact) = tag_hash(&item.key);
			let t = (h >> 56) as u8;
			// Memo hit: the preceding get confirmed this exact key absent and
			// nothing has mutated the bucket since, so skip the tag scan.
			let known_absent = exact && *self.miss_valid.get_mut() && *self.miss_hash.get_mut() == h;
			// Every put mutates the bucket: invalidate before any return.
			self.clear_memo();
			if !known_absent {
				if let Some(position) = self.get_position_tagged(&item.key, t) {
					self.items[position].value = item.value;
					let pushed_item = self.get_from_position(position);
					return (pushed_item, None);
				}
			}
			debug_assert!(
				self.items.iter().all(|it| it.key != item.key),
				"miss memo skipped the scan for a resident key"
			);

			// `clock` is the BIP miss counter here; no timestamp is written.
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

	fn memo_valid(b: &AssociativeCacheBucket<i64, i64>) -> bool {
		b.miss_valid.load(Ordering::Relaxed)
	}

	/// Layout: Vec (24) + Vec (24) + usize (8) + Box<dyn> (16) + AtomicU64 (8)
	/// + AtomicBool + 2 x bool (3) + u32 (4) = 87 bytes, padded to 8-byte
	/// alignment = 88. The stride grows from the parent's 80 deliberately: the
	/// `miss_hash` memo word is the change under test (`miss_valid` fits in
	/// the parent's padding).
	#[cfg(target_pointer_width = "64")]
	#[test]
	fn bucket_size_matches_parent() {
		assert_eq!(std::mem::size_of::<AssociativeCacheBucket<u64, u64>>(), 88);
	}

	#[test]
	fn integer_keys_hash_exactly() {
		assert!(tag_hash(&42i64).1);
		assert!(tag_hash(&42u64).1);
		assert!(tag_hash(&42u32).1);
		// Multi-write / byte-slice keys are never treated as exact.
		assert!(!tag_hash(&(1i64, 2i64)).1);
		assert!(!tag_hash(&"abc").1);
		assert!(!tag_hash(&7u8).1);
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

	/// Every miss is a get followed by a put of the same key, so each put goes
	/// through the miss memo; order and evictions must still match the model.
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
					// The get miss must have armed the memo for this key.
					assert!(memo_valid(&b));
					assert_eq!(b.miss_hash.load(Ordering::Relaxed), tag_hash(&key).0);
					let (ins, ev) = b.put(Item::new(key, key * 2));
					assert_eq!(ins.key, key);
					assert_eq!(ins.value, key * 2);
					let ev_r = r.put(key);
					assert_eq!(ev.map(|i| i.key), ev_r);
					assert!(!memo_valid(&b), "put must clear the memo");
				}
				let order: Vec<i64> = b.items.iter().map(|i| i.key).collect();
				assert_eq!(order, r.keys, "order diverged at size {size}");
				check(&b);
				assert!(b.size() <= size);
			}
		}
	}

	#[test]
	fn memo_never_duplicates() {
		for &size in &[1usize, 2, 4, 37] {
			let mut b = bucket(size);
			// get-miss A arms the memo for A; put B must clear it, so the
			// later put A cannot be confused (A is absent here, insert once).
			assert!(b.get(&100).is_none());
			assert!(memo_valid(&b));
			b.put(Item::new(200, 2));
			assert!(!memo_valid(&b));
			b.put(Item::new(100, 1));
			assert_eq!(b.items.iter().filter(|i| i.key == 100).count(), 1);
			// Now A is resident: a stale memo must never skip its lookup.
			assert!(b.get(&300).is_none());
			assert!(memo_valid(&b));
			let before = b.size();
			b.put(Item::new(100, 11));
			assert_eq!(b.size(), before, "update of resident key must not insert");
			assert_eq!(b.items.iter().filter(|i| i.key == 100).count(), 1);
			assert_eq!(b.get(&100).map(|i| i.value), Some(11));
			check(&b);
		}

		// get_items_mut clears the memo.
		let mut b = bucket(4);
		assert!(b.get(&5).is_none());
		assert!(memo_valid(&b));
		let _ = b.get_items_mut();
		assert!(!memo_valid(&b));

		// evict_item and remove_at clear it too.
		b.put(Item::new(1, 1));
		b.put(Item::new(2, 2));
		assert!(b.get(&9).is_none());
		assert!(memo_valid(&b));
		b.remove_at(0);
		assert!(!memo_valid(&b));
		assert!(b.get(&9).is_none());
		assert!(memo_valid(&b));
		b.evict_item();
		assert!(!memo_valid(&b));
		check(&b);

		// A get hit does not arm the memo.
		b.put(Item::new(3, 3));
		assert!(b.get(&3).is_some());
		assert!(!memo_valid(&b));
	}
}
