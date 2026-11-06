use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::traits::eviction_policy::EvictionPolicy;
use crate::shared::utils::hash;
use std::{
	fmt::{Debug, Display, Formatter},
	hash::Hash,
};

/// Slot capacity of the inline fingerprint array. Buckets with a larger
/// `max_size` never enable fingerprints (they fall back to the plain scan).
pub(crate) const FP_CAP: usize = 64;

const LO: u64 = 0x0101_0101_0101_0101;
const HI: u64 = 0x8080_8080_8080_8080;

/// 1-byte fingerprint of a key from its `utils::hash` value: the top 8 bits
/// (independent of the low bits the bucket indexes use), with bit 0 forced on
/// so that 0 always means "empty slot".
#[inline]
pub(crate) fn fp_from_hash(h: usize) -> u8 {
	(((h as u64) >> 56) as u8) | 1
}

/// A bucket of items ordered per its eviction policy.
///
/// Stash buckets (cycle 8 H02) additionally carry a per-slot fingerprint array
/// `fps`: `fps[i] == fp(items[i].key)` for every occupied slot and `0` for every
/// empty slot. Every in-bucket mutation (`put_at`, `remove_at`,
/// `get_from_position`) applies the same shift/rotate to `fps` that it applies to
/// `items`, so lookups compare fingerprints first and only key-compare slots
/// whose fingerprint matches. Normal buckets leave `fps` unused.
///
/// `get_items_mut` hands out the raw `Vec`, so it marks the fingerprints stale;
/// lookups then use the plain scan and the next mutation rebuilds them.
#[derive(Debug)]
#[repr(C, align(64))]
pub struct DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Per-slot fingerprints, placed first so they share the bucket header's
	/// (64-byte aligned) allocation.
	fps: [u8; FP_CAP],
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Whether this bucket maintains fingerprints at all (stash buckets only).
	fingerprinted: bool,
	/// `fps` currently describes `items` exactly. Only ever true when `fingerprinted`.
	fps_valid: bool,
}

impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// A plain (non-fingerprinted) bucket, used for normal buckets.
	pub fn new(max_size: usize, eviction_policy: Box<dyn EvictionPolicy<K, V>>) -> Self {
		Self::with_fingerprints(max_size, eviction_policy, false)
	}

	/// A fingerprinted bucket, used for stash buckets. Fingerprints are enabled
	/// only when `max_size <= FP_CAP`.
	pub fn new_stash(max_size: usize, eviction_policy: Box<dyn EvictionPolicy<K, V>>) -> Self {
		Self::with_fingerprints(max_size, eviction_policy, max_size <= FP_CAP)
	}

	fn with_fingerprints(max_size: usize, eviction_policy: Box<dyn EvictionPolicy<K, V>>, fp: bool) -> Self {
		DashBucket {
			fps: [0u8; FP_CAP],
			items: Vec::with_capacity(max_size),
			max_size,
			eviction_policy,
			fingerprinted: fp,
			fps_valid: fp,
		}
	}

	/// Rebuilds stale fingerprints (only after a raw `get_items_mut` access).
	#[inline]
	fn ensure_fps(&mut self) {
		if self.fingerprinted && !self.fps_valid {
			self.rebuild_fps();
		}
	}

	#[cold]
	fn rebuild_fps(&mut self) {
		let len = self.items.len();
		if len > FP_CAP {
			return;
		}
		for i in 0..len {
			self.fps[i] = fp_from_hash(hash(&self.items[i].key));
		}
		for b in &mut self.fps[len..] {
			*b = 0;
		}
		self.fps_valid = true;
	}

	/// Fingerprint-filtered lookup. Requires `fps_valid`.
	#[inline]
	fn fp_position(&self, key: &K, f: u8) -> Option<usize> {
		let len = self.items.len();
		let pat = LO.wrapping_mul(f as u64);
		let words = len.div_ceil(8);
		for w in 0..words {
			let base = w * 8;
			let mut bytes = [0u8; 8];
			bytes.copy_from_slice(&self.fps[base..base + 8]);
			let x = u64::from_le_bytes(bytes) ^ pat;
			// Flags every byte of `x` that is zero (no false negatives; possible
			// false positives above a true match, filtered by the key compare).
			let mut m = x.wrapping_sub(LO) & !x & HI;
			while m != 0 {
				let i = base + ((m.trailing_zeros() as usize) >> 3);
				if i >= len {
					break;
				}
				if self.items[i].key == *key {
					return Some(i);
				}
				m &= m - 1;
			}
		}
		None
	}

	#[inline]
	fn plain_position(&self, key: &K) -> Option<usize> {
		self.items.iter().position(|d| d.key == *key)
	}

	/// Every occupied slot's fingerprint matches its key; every empty slot is 0.
	fn fps_consistent(&self) -> bool {
		if !self.fps_valid {
			return true;
		}
		let len = self.items.len();
		len <= FP_CAP
			&& (0..len).all(|i| self.fps[i] == fp_from_hash(hash(&self.items[i].key)))
			&& self.fps[len..].iter().all(|&b| b == 0)
	}

	#[inline]
	fn debug_check_fps(&self) {
		debug_assert!(self.fps_consistent(), "DashBucket: fingerprints out of sync with items");
	}
}

pub fn new_bucket<K, V>(
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
) -> Box<dyn Bucket<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	Box::new(DashBucket::new(max_size, eviction_policy))
}

/// A stash bucket: a [`DashBucket`] with per-slot fingerprints enabled.
pub fn new_stash_bucket<K, V>(
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
) -> Box<dyn Bucket<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	Box::new(DashBucket::new_stash(max_size, eviction_policy))
}

impl<K, V> Bucket<K, V> for DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		// Raw access may reorder/replace items: fingerprints become stale.
		self.fps_valid = false;
		&mut self.items
	}

	fn get_item_mut(&mut self, index: usize) -> &mut Item<K, V> {
		// Metadata-only access (key unchanged): fingerprints stay valid.
		&mut self.items[index]
	}

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	fn get_position(&self, key: &K) -> Option<usize> {
		if self.fps_valid {
			self.fp_position(key, fp_from_hash(hash(key)))
		} else {
			self.plain_position(key)
		}
	}

	fn get_position_hashed(&self, key: &K, h: usize) -> Option<usize> {
		let pos = if self.fps_valid {
			self.fp_position(key, fp_from_hash(h))
		} else {
			self.plain_position(key)
		};
		debug_assert_eq!(pos, self.plain_position(key), "fingerprint lookup diverged from plain scan");
		pos
	}

	fn put_at(&mut self, item: Item<K, V>, pos: InsertPosition) -> (&Item<K, V>, Option<Item<K, V>>) {
		let h = if self.fingerprinted { hash(&item.key) } else { 0 };
		self.put_at_hashed(item, pos, h)
	}

	fn put_at_hashed(
		&mut self,
		item: Item<K, V>,
		pos: InsertPosition,
		h: usize,
	) -> (&Item<K, V>, Option<Item<K, V>>) {
		self.ensure_fps();
		if let Some(position) = self.get_position_hashed(&item.key, h) {
			log::trace!("DashBucket.put_at: key={:?} update_existing", item.key);
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		let len = self.items.len();
		if self.fps_valid && len >= FP_CAP {
			self.fps_valid = false;
		}
		let f = fp_from_hash(h);
		let idx = match pos {
			InsertPosition::Back => {
				self.items.push(item);
				if self.fps_valid {
					self.fps[len] = f;
				}
				len
			}
			InsertPosition::Front => {
				self.items.insert(0, item);
				if self.fps_valid {
					self.fps.copy_within(0..len, 1);
					self.fps[0] = f;
				}
				0
			}
		};
		self.debug_check_fps();
		(&self.items[idx], evicted_item)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		self.ensure_fps();
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		if self.fps_valid && index != position {
			if index > position {
				// Move-to-back (ClassicLRU): items[position..=index] rotated left by one.
				self.fps[position..=index].rotate_left(1);
			} else {
				// Swap toward front (Swap policy).
				self.fps.swap(position, index);
			}
		}
		self.debug_check_fps();
		&self.items[index]
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.ensure_fps();
		let len = self.items.len();
		let removed = self.items.remove(index);
		if self.fps_valid {
			self.fps.copy_within(index + 1..len, index);
			self.fps[len - 1] = 0;
		}
		self.debug_check_fps();
		removed
	}
}

impl<K, V> Display for DashBucket<K, V>
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
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;

	fn xorshift(x: &mut u64) -> u64 {
		*x ^= *x << 13;
		*x ^= *x >> 7;
		*x ^= *x << 17;
		*x
	}

	/// Fingerprinted and plain buckets behave identically under every policy, and
	/// fingerprints stay exact after every mutation.
	#[test]
	fn fingerprinted_bucket_matches_plain_bucket() {
		// TimestampLRU is excluded: two independent buckets get independent
		// `Instant::now()` stamps, so tie-breaking could legitimately differ.
		for policy in EvictionPolicyKind::all()
			.into_iter()
			.filter(|p| *p != EvictionPolicyKind::TimestampLRU)
		{
			let mut fp_b = DashBucket::<i64, i64>::new_stash(8, policy.build());
			let mut plain = DashBucket::<i64, i64>::new(8, policy.build());
			assert!(fp_b.fps_valid);
			let mut x = 0x9E37_79B9_7F4A_7C15_u64;
			for _ in 0..4000 {
				let r = xorshift(&mut x);
				let key = (r % 40) as i64;
				match r % 5 {
					0 | 1 => {
						let a = fp_b.get_position(&key);
						assert_eq!(a, plain.get_position(&key), "policy={policy:?}");
						if let Some(p) = a {
							fp_b.get_from_position(p);
							plain.get_from_position(p);
						}
					}
					2 => {
						let pos = if r & 64 == 0 { InsertPosition::Back } else { InsertPosition::Front };
						let (_, e1) = fp_b.put_at(Item::new(key, key), pos);
						let (_, e2) = plain.put_at(Item::new(key, key), pos);
						assert_eq!(e1.map(|i| i.key), e2.map(|i| i.key));
					}
					3 => {
						let h = hash(&key);
						assert_eq!(fp_b.get_position_hashed(&key, h), plain.get_position(&key));
					}
					_ => {
						if !fp_b.items.is_empty() {
							let i = (r as usize >> 8) % fp_b.items.len();
							assert_eq!(fp_b.remove_at(i).key, plain.remove_at(i).key);
						}
					}
				}
				assert!(fp_b.fps_valid);
				assert!(fp_b.fps_consistent());
				let k1: Vec<i64> = fp_b.items.iter().map(|i| i.key).collect();
				let k2: Vec<i64> = plain.items.iter().map(|i| i.key).collect();
				assert_eq!(k1, k2, "policy={policy:?}");
			}
		}
	}

	#[test]
	fn raw_items_access_falls_back_and_rebuilds() {
		let mut b = DashBucket::<i64, i64>::new_stash(4, EvictionPolicyKind::ClassicLRU.build());
		b.put(Item::new(1, 10));
		b.get_items_mut().push(Item::new(2, 20));
		assert!(!b.fps_valid);
		assert_eq!(b.get_position(&2), Some(1));
		b.put(Item::new(3, 30));
		assert!(b.fps_valid);
		assert!(b.fps_consistent());
		assert_eq!(b.get_position(&2), Some(1));
		assert_eq!(b.get_position(&3), Some(2));
	}

	#[test]
	fn empty_slots_are_zero_and_fp_never_zero() {
		let b = DashBucket::<i64, i64>::new_stash(4, EvictionPolicyKind::ClassicLRU.build());
		assert!(b.fps.iter().all(|&f| f == 0));
		for k in 0..1000_i64 {
			assert_ne!(fp_from_hash(hash(&k)), 0);
		}
	}

	#[test]
	fn item_stays_40_bytes() {
		assert_eq!(std::mem::size_of::<Item<i64, i64>>(), 40);
	}
}
