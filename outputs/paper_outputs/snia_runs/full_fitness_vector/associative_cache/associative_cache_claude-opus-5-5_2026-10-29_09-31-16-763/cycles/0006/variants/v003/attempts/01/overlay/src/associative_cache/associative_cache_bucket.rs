use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::debug;

use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// ClassicLRU insertion depth: a newly admitted item is inserted at
/// `len / INSERT_DEPTH_DIVISOR` (measured from the eviction end, index 0)
/// instead of at the MRU end. A hit still promotes it to the back.
const INSERT_DEPTH_DIVISOR: usize = 4;

/// Bimodal insertion (BIP): every `BIP_MRU_PERIOD`-th new-key put in a bucket
/// is inserted at the MRU end instead of at quarter depth.
const BIP_MRU_PERIOD: u32 = 32;

/// ClassicLRU second chance: at most this many LRU-end items with their reuse
/// bit set are spared (bit cleared, rotated to quarter depth) per eviction.
const SECOND_CHANCE_MAX: usize = 2;

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
	/// On this path `Item::lfu_counter` is a 0/1 reuse bit owned solely by
	/// the bucket (ClassicLRU never touches it).
	lip: bool,
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
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			tags: if lip { Vec::with_capacity(settings.bucket_size) } else { Vec::new() },
			max_size: settings.bucket_size,
			eviction_policy,
			lip,
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

	/// Bounded second chance (lip path only): while the LRU-end item carries
	/// its reuse bit, clear it and rotate it to quarter depth (probationary
	/// region), at most `SECOND_CHANCE_MAX` times. `tags` is rotated in lockstep.
	#[inline]
	fn second_chance(&mut self) {
		let len = self.items.len();
		if len == 0 {
			return;
		}
		let q = (len - 1) / INSERT_DEPTH_DIVISOR;
		let mut spared = 0;
		while spared < SECOND_CHANCE_MAX && self.items[0].lfu_counter != 0 {
			self.items[0].lfu_counter = 0;
			if q > 0 {
				self.items[..=q].rotate_left(1);
				self.tags[..=q].rotate_left(1);
			}
			spared += 1;
		}
		self.debug_check_tags();
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
			// Grow: mark the item as reused (second-chance bit).
			self.items[index].lfu_counter = 1;
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

			let evicted_item = if self.is_full() {
				// Decay: spare up to SECOND_CHANCE_MAX reused LRU-end items, then
				// evict whatever is at index 0 (bounded work per miss).
				self.second_chance();
				self.evict_item()
			} else {
				None
			};

			self.bip_tick = self.bip_tick.wrapping_add(1);
			if self.bip_tick % BIP_MRU_PERIOD == 0 {
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
		let mut keys: Vec<i64> = b.items.iter().map(|i| i.key).collect();
		keys.sort();
		keys.dedup();
		assert_eq!(keys.len(), b.items.len(), "duplicate keys");
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
	fn tags_stay_in_lockstep_with_second_chance() {
		// Mix of a hot set (frequently hit, so reuse bits are set) and a scan,
		// so spare-rotations are exercised on most evictions.
		for size in [1usize, 2, 3, 4, 5, 8, 37, 64] {
			let mut b = bucket(size);
			for k in 0..5000i64 {
				let key = if k % 3 == 0 { k % 7 } else { 1000 + k };
				let hit = b.get(&key).map(|it| it.value);
				match hit {
					Some(v) => assert_eq!(v, key),
					None => {
						b.put(Item::new(key, key));
					}
				}
				check(&b);
				assert!(b.size() <= size);
				assert!(b.items.iter().all(|i| i.lfu_counter <= 1));
			}
		}
	}

	#[test]
	fn reused_lru_item_gets_second_chance() {
		let mut b = bucket(8);
		for k in 0..8i64 {
			b.put(Item::new(k, k));
		}
		// Hit the current LRU-end item so it carries the reuse bit, then
		// age it back to index 0 by hitting everything else.
		let lru = b.items[0].key;
		assert!(b.get(&lru).is_some());
		let others: Vec<i64> = b.items.iter().map(|i| i.key).filter(|k| *k != lru).collect();
		for k in others {
			assert!(b.get(&k).is_some());
		}
		assert_eq!(b.items[0].key, lru);
		// Clear the others' bits so only `lru` is spared.
		for it in b.items.iter_mut().skip(1) {
			it.lfu_counter = 0;
		}
		let (_, evicted) = b.put(Item::new(100, 100));
		assert_ne!(evicted.expect("full bucket evicts").key, lru);
		assert!(b.get_position(&lru).is_some());
		check(&b);
	}
}
