//! H03: paired bucket.
//!
//! One physical bucket holds a *pair* of logical buckets ("halves"), each
//! `half_size` slots, so the physical bucket holds `2 * half_size`. That keeps
//! the total capacity the same as the baseline's `n` buckets of `bucket_size`.
//! A key's home half comes from its hash. The other half is its partner. Items
//! live in one `Vec`: half 0 is `items[0..len0]` and half 1 is `items[len0..]`.
//! Each half is ordered by the eviction policy on its own.
//!
//! - get: probe home first. On a partner hit, promote the item home, and swap
//!   home's eviction candidate into the slot the item vacated in the partner.
//! - put: insert at home. If that evicts a native item, demote it to the
//!   partner when there is a free slot or it is newer than the partner's
//!   eviction candidate. An evicted guest is dropped.
//!
//! A physical bucket with no partner (the last one when the logical bucket
//! count is odd) behaves like a single baseline bucket.
//!
//! Cycle 2: item timestamps are 32-bit logical ticks. The demotion's "newer
//! than" check is wrap-safe (`crate::shared::item::is_newer`).

use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::{is_newer, Item};
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use crate::shared::utils::hash;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Number of items in half 0 (`items[0..len0]`).
	len0: usize,
	/// Slots per half (the logical bucket size).
	half_size: usize,
	/// Total slots in this physical bucket.
	max_size: usize,
	/// Whether this bucket has two halves (false: one half of `half_size`).
	paired: bool,
	/// Number of physical buckets. Used to pick the home half from the hash.
	divisor: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// A single (unpaired) bucket of `settings.bucket_size` slots.
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self::with_layout(settings, false, 1)
	}

	/// A paired (two halves of `bucket_size`) or unpaired bucket. `divisor` is
	/// the number of physical buckets in the cache.
	pub fn with_layout(settings: &AssociativeCacheSettings, paired: bool, divisor: usize) -> Self {
		let half_size = settings.bucket_size;
		let max_size = if paired { 2 * half_size } else { half_size };
		Self {
			items: Vec::with_capacity(max_size),
			len0: 0,
			half_size,
			max_size,
			paired,
			divisor: divisor.max(1),
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	/// Home half for a key whose full hash is `h`.
	#[inline]
	pub fn home_half_of_hash(&self, h: usize) -> usize {
		if self.paired {
			(h / self.divisor) & 1
		} else {
			0
		}
	}

	#[inline]
	fn home_half_of_key(&self, key: &K) -> usize {
		if self.paired {
			self.home_half_of_hash(hash(key))
		} else {
			0
		}
	}

	#[inline]
	fn range(&self, half: usize) -> (usize, usize) {
		if half == 0 {
			(0, self.len0)
		} else {
			(self.len0, self.items.len())
		}
	}

	#[inline]
	fn half_len(&self, half: usize) -> usize {
		let (s, e) = self.range(half);
		e - s
	}

	#[inline]
	fn half_of_position(&self, position: usize) -> usize {
		if position < self.len0 {
			0
		} else {
			1
		}
	}

	#[inline]
	fn find_in(&self, half: usize, key: &K) -> Option<usize> {
		let (s, e) = self.range(half);
		self.items[s..e].iter().position(|d| d.key == *key).map(|i| s + i)
	}

	/// Applies the policy's access side effect within the half holding
	/// `position`. Returns the item's new absolute index.
	#[inline]
	fn touch(&mut self, position: usize) -> usize {
		let (s, e) = self.range(self.half_of_position(position));
		s + self.eviction_policy.get_from_position_in_slice(&mut self.items[s..e], position - s)
	}

	/// Appends `item` to the back of `half`. Returns its absolute index.
	#[inline]
	fn push_half(&mut self, half: usize, item: Item<K, V>) -> usize {
		if half == 0 {
			self.items.insert(self.len0, item);
			self.len0 += 1;
			self.len0 - 1
		} else {
			self.items.push(item);
			self.items.len() - 1
		}
	}

	/// Inserts `item` into `half` at offset `rel` within that half.
	#[inline]
	fn insert_half_at(&mut self, half: usize, rel: usize, item: Item<K, V>) {
		let (s, e) = self.range(half);
		let abs = (s + rel).min(e);
		self.items.insert(abs, item);
		if half == 0 {
			self.len0 += 1;
		}
	}

	/// Absolute index of `half`'s eviction candidate, if the half is non-empty.
	#[inline]
	fn candidate_in(&self, half: usize) -> Option<usize> {
		let (s, e) = self.range(half);
		if s == e {
			return None;
		}
		Some(s + self.eviction_policy.eviction_candidate_index(&self.items[s..e]))
	}

	#[inline]
	fn evict_from(&mut self, half: usize) -> Option<Item<K, V>> {
		let c = self.candidate_in(half)?;
		Some(self.remove_at(c))
	}

	/// Paired lookup. `h` is the key's full hash. Probes home, then the partner.
	/// A partner hit promotes the item home and swaps home's eviction candidate
	/// into the vacated partner slot.
	pub fn pair_get(&mut self, key: &K, h: usize) -> Option<&Item<K, V>> {
		let home = self.home_half_of_hash(h);
		if let Some(i) = self.find_in(home, key) {
			let j = self.touch(i);
			return Some(&self.items[j]);
		}
		if !self.paired {
			return None;
		}
		let partner = 1 - home;
		let j = self.find_in(partner, key)?;
		let rel = j - self.range(partner).0;
		let item = self.remove_at(j);
		if self.half_len(home) >= self.half_size {
			// Swap, not drop: home's eviction candidate takes the vacated
			// partner slot. It is either native to home (it becomes a guest in
			// its partner) or a guest from the partner (it goes back home).
			// Either way it stays in one of its two legal halves.
			if let Some(x) = self.evict_from(home) {
				self.insert_half_at(partner, rel, x);
			}
		}
		let pos = self.push_half(home, item);
		let k = self.touch(pos);
		Some(&self.items[k])
	}

	/// Paired insert. `h` is the key's full hash. Updates in place if the key
	/// is in either half. Returns the item dropped from the bucket, if any.
	pub fn pair_put(&mut self, item: Item<K, V>, h: usize) -> Option<Item<K, V>> {
		let home = self.home_half_of_hash(h);
		let existing = self
			.find_in(home, &item.key)
			.or_else(|| if self.paired { self.find_in(1 - home, &item.key) } else { None });
		if let Some(p) = existing {
			self.items[p].value = item.value;
			self.touch(p);
			return None;
		}

		let evicted = if self.half_len(home) >= self.half_size { self.evict_from(home) } else { None };
		self.push_half(home, item);

		let e = evicted?;
		if !self.paired || self.home_half_of_key(&e.key) != home {
			// No partner, or `e` was already a guest here: drop it.
			return Some(e);
		}
		let partner = 1 - home;
		if self.half_len(partner) < self.half_size {
			self.push_half(partner, e);
			return None;
		}
		let c = self.candidate_in(partner).expect("full partner is non-empty");
		// Wrap-safe logical-tick comparison: valid while stamps are < 2^31 apart.
		if is_newer(e.timestamp, self.items[c].timestamp) {
			let dropped = self.remove_at(c);
			self.push_half(partner, e);
			Some(dropped)
		} else {
			Some(e)
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
		let key = item.key;
		let h = hash(&key);
		let dropped = self.pair_put(item, h);
		let pos = self.get_position(&key).expect("bucket invariant: just inserted");
		(&self.items[pos], dropped)
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let h = hash(key);
		self.pair_get(key, h)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.touch(position);
		&self.items[index]
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
		self.items.remove(index)
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for (i, item) in self.items.iter().enumerate() {
			let half = if i < self.len0 { 0 } else { 1 };
			writeln!(f, "    [{half}] {}", item)?;
		}
		Ok(())
	}
}
