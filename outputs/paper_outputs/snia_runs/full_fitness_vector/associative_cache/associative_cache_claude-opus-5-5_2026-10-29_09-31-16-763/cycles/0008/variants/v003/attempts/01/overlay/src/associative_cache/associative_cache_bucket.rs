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

/// Width of one SWAR tag-scan word (8 tags per `u64` load).
const SWAR_WIDTH: usize = 8;
/// Every byte = 0x01; multiplying a byte by it broadcasts it to all 8 lanes.
const SWAR_ONES: u64 = 0x0101_0101_0101_0101;
/// Every byte = 0x7f (low 7 bits of each lane).
const SWAR_LOW7: u64 = 0x7f7f_7f7f_7f7f_7f7f;

/// Exact zero-byte mask: bit 7 of each byte is set iff that byte of `x` is 0;
/// every other bit is clear. Carry-free (each lane's add stays below 0x100),
/// so there are no false flags, unlike the classic `(x - 0x01..) & !x & 0x80..`.
#[inline(always)]
fn swar_zero_bytes(x: u64) -> u64 {
	let y = (x & SWAR_LOW7).wrapping_add(SWAR_LOW7);
	!(y | x | SWAR_LOW7)
}

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

	/// Tag-filtered lookup (lip path only). SWAR scan: loads 8 tags per `u64`,
	/// XORs with the broadcast probe tag and extracts an exact zero-byte mask,
	/// then full-compares only candidates. Candidates are visited in ascending
	/// index order (little-endian load + `trailing_zeros`), so the first match
	/// returned is the same as a forward scalar scan's.
	#[inline]
	fn get_position_tagged(&self, key: &K, t: u8) -> Option<usize> {
		debug_assert_eq!(self.tags.len(), self.items.len());
		let tags = &self.tags[..];
		let items = &self.items[..tags.len()];
		let pat = (t as u64).wrapping_mul(SWAR_ONES);
		let mut base = 0usize;
		let mut chunks = tags.chunks_exact(SWAR_WIDTH);
		for c in &mut chunks {
			let mut buf = [0u8; SWAR_WIDTH];
			buf.copy_from_slice(c);
			let word = u64::from_le_bytes(buf);
			let mut m = swar_zero_bytes(word ^ pat);
			while m != 0 {
				let i = base + (m.trailing_zeros() / 8) as usize;
				if items[i].key == *key {
					return Some(i);
				}
				m &= m - 1;
			}
			base += SWAR_WIDTH;
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

	fn put(&mut self, mut item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if self.lip {
			// Hash once per op; reuse the tag for lookup and insertion.
			let t = tag_of(&item.key);
			if let Some(position) = self.get_position_tagged(&item.key, t) {
				self.items[position].value = item.value;
				let pushed_item = self.get_from_position(position);
				return (pushed_item, None);
			}

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

	/// Scalar reference for `get_position_tagged`: first ascending index whose
	/// tag equals `t` and whose key equals `key`.
	fn scalar_position(b: &AssociativeCacheBucket<i64, i64>, key: &i64, t: u8) -> Option<usize> {
		(0..b.tags.len()).find(|&i| b.tags[i] == t && b.items[i].key == *key)
	}

	#[test]
	fn swar_zero_bytes_is_exact() {
		// Every single-byte value in every lane, against varied backgrounds.
		for lane in 0..8u32 {
			for v in 0..=255u64 {
				for &bg in &[0u64, u64::MAX, 0x8080_8080_8080_8080, 0x0101_0101_0101_0101, 0x7f00_ff01_8000_017f] {
					let x = (bg & !(0xffu64 << (lane * 8))) | (v << (lane * 8));
					let m = swar_zero_bytes(x);
					let mut expect = 0u64;
					for j in 0..8 {
						if (x >> (j * 8)) & 0xff == 0 {
							expect |= 0x80u64 << (j * 8);
						}
					}
					assert_eq!(m, expect, "x={x:#018x}");
				}
			}
		}
	}

	#[test]
	fn swar_scan_matches_scalar_with_forced_collisions() {
		let mut s: u64 = 0xDEAD_BEEF_CAFE_F00D;
		let mut next = || {
			s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			s >> 17
		};
		let sizes = [0usize, 1, 7, 8, 9, 13, 15, 16, 17, 64, 255, 300, 1024, 1029];
		for &n in &sizes {
			// Mode 0: genuine tags of distinct keys that all share one tag value
			// (maximal collisions). Mode 1: tags drawn from a 3-value alphabet,
			// with duplicate keys, so ascending tie-breaking is exercised.
			for mode in 0..2 {
				let mut b = bucket(n.max(1));
				let target = 0x5Au8;
				let mut k = 0i64;
				for i in 0..n {
					let (key, t) = if mode == 0 {
						while tag_of(&k) != target {
							k += 1;
						}
						let key = k;
						k += 1;
						(key, target)
					} else {
						let key = (next() % (n as u64 / 2 + 1)) as i64;
						let t = [0u8, 0x80, target][(next() % 3) as usize];
						(key, t)
					};
					let _ = i;
					b.items.push(Item::new(key, key));
					b.tags.push(t);
				}
				let mut probes: Vec<(i64, u8)> = Vec::new();
				for it in &b.items {
					probes.push((it.key, tag_of(&it.key)));
					for &t in &[0u8, 0x80, target, 0xff, 0x7f, 0x01] {
						probes.push((it.key, t));
					}
				}
				// Absent keys, including ones sharing the collided tag.
				for a in [-1i64, -2, i64::MIN, i64::MAX, 1 << 40] {
					for &t in &[0u8, 0x80, target, tag_of(&a)] {
						probes.push((a, t));
					}
				}
				for (key, t) in probes {
					assert_eq!(
						b.get_position_tagged(&key, t),
						scalar_position(&b, &key, t),
						"n={n} mode={mode} key={key} t={t:#04x}"
					);
				}
				if mode == 0 {
					check(&b);
					for (i, it) in b.items.iter().enumerate() {
						assert_eq!(b.get_position(&it.key), Some(i));
					}
					for a in [-1i64, -2, i64::MIN, i64::MAX] {
						assert_eq!(b.get_position(&a), None);
					}
				}
			}
		}
	}

	/// Parent layout: Vec (24) + Vec (24) + usize (8) + Box<dyn> (16) +
	/// 2 x bool (2) + u32 (4) = 78 bytes, padded to 8-byte alignment = 80.
	/// Renaming `bip_tick` to `clock` must not change the bucket stride.
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

	#[test]
	fn fused_matches_reference_remove_insert() {
		for &size in &[1usize, 2, 3, 4, 5, 8, 13, 16, 17, 37, 64] {
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
