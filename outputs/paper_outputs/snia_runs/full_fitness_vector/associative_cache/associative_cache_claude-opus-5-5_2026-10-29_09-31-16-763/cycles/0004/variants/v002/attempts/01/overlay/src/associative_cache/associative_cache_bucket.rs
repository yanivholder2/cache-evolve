use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// ClassicLRU insertion depth: a newly admitted item is inserted at
/// `len / INSERT_DEPTH_DIVISOR` (measured from the eviction end, index 0)
/// instead of at the MRU end. A hit still promotes it to the back.
const INSERT_DEPTH_DIVISOR: usize = 4;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// True when the policy is ClassicLRU (decided once at construction).
	lip: bool,
	/// Hashed "recently evicted" bit set (~2 bits per slot). Empty unless `lip`.
	ghost_bits: Vec<u64>,
	/// Power-of-two bit count of `ghost_bits` minus 1.
	ghost_mask: usize,
	/// Evictions recorded since the last bit-set reset (coarse aging).
	ghost_evictions: u32,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let eviction_policy: Box<dyn EvictionPolicy<K, V>> = settings.eviction_policy.build();
		let lip = eviction_policy.name() == "ClassicLRU";
		let max_size = settings.bucket_size;
		let (ghost_bits, ghost_mask) = if lip {
			let ghost_words = std::cmp::max(1, (2 * max_size + 63) / 64);
			let bits = ghost_words * 64;
			// Round down to a power of two.
			let pow2 = 1usize << (usize::BITS - 1 - bits.leading_zeros());
			(vec![0u64; ghost_words], pow2 - 1)
		} else {
			(Vec::new(), 0)
		};
		Self {
			items: Vec::with_capacity(max_size),
			max_size,
			eviction_policy,
			lip,
			ghost_bits,
			ghost_mask,
			ghost_evictions: 0,
		}
	}

	#[inline]
	fn ghost_index(&self, key: &K) -> usize {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let key_hash = hasher.finish();
		let h = key_hash.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32;
		(h as usize) & self.ghost_mask
	}

	/// Tests and clears the ghost bit for `key`; returns whether it was set.
	#[inline]
	fn ghost_take(&mut self, key: &K) -> bool {
		let idx = self.ghost_index(key);
		let (w, b) = (idx >> 6, 1u64 << (idx & 63));
		let set = self.ghost_bits[w] & b != 0;
		if set {
			self.ghost_bits[w] &= !b;
		}
		set
	}

	/// Records `key` as recently evicted, aging the filter periodically.
	#[inline]
	fn ghost_record(&mut self, key: &K) {
		let idx = self.ghost_index(key);
		self.ghost_bits[idx >> 6] |= 1u64 << (idx & 63);
		self.ghost_evictions += 1;
		if self.ghost_evictions as usize >= 2 * self.max_size {
			for w in self.ghost_bits.iter_mut() {
				*w = 0;
			}
			self.ghost_evictions = 0;
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

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing key: identical to the trait default.
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		if self.lip {
			// (a) Was this key evicted recently? Then it is being reused.
			let reuse = self.ghost_take(&item.key);

			// (b) Evict if full and remember the victim in the ghost filter.
			let evicted_item = if self.is_full() { self.evict_item() } else { None };
			if let Some(ev) = evicted_item.as_ref() {
				let ev_key = ev.key;
				self.ghost_record(&ev_key);
			}

			// (c) Reused keys go to the MRU end; others to quarter depth.
			if reuse {
				self.items.push(item);
				let pushed_item = self.items.last().expect("bucket invariant: just pushed");
				return (pushed_item, evicted_item);
			}
			let insert_pos = self.items.len() / INSERT_DEPTH_DIVISOR;
			self.items.insert(insert_pos, item);
			return (&self.items[insert_pos], evicted_item);
		}

		let evicted_item = if self.is_full() { self.evict_item() } else { None };

		// All other policies: default behaviour (append at the back).
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
