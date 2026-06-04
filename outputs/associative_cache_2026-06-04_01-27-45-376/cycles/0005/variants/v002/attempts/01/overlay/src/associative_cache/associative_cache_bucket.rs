use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::cell::Cell;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Per-slot CLOCK reference bitmask.
	///
	/// Bit `i` is `1` if the item currently at slot `i` has been referenced
	/// (touched by `get`) since the last mass-clear, and `0` if it has been
	/// untouched. Only consulted under `ClassicLRU`: it drives the
	/// untouched-first eviction sweep and is mass-cleared the moment every
	/// slot is touched, restoring the untouched-vs-touched signal. For every
	/// other policy the bits are maintained in step with item permutations
	/// but never read (eviction delegates to the policy).
	///
	/// Invariant: bits at positions `>= items.len()` are always `0`.
	///
	/// Interior mutability is required because `eviction_candidate_index`
	/// borrows `&self` via the trait yet may mass-clear the bitmask on the
	/// all-touched sweep.
	///
	/// `u16` covers `bucket_size <= 16` (the benchmark geometry: 2048 / 128).
	ref_bits: Cell<u16>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		debug_assert!(
			settings.bucket_size <= 16,
			"AssociativeCacheBucket: ref_bits is u16, bucket_size must be <= 16 (got {})",
			settings.bucket_size
		);
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			ref_bits: Cell::new(0),
		}
	}

	/// Bits at positions `0..position` (unchanged) ORed with bits at
	/// `position+1..len` shifted down by one. Used to mirror `Vec::remove`
	/// (and `Vec`'s `remove`+`push` combo under `ClassicLRU`) on `ref_bits`.
	#[inline]
	fn collapse_at(bits: u16, position: usize, len: usize) -> u16 {
		debug_assert!(position < len);
		debug_assert!(len <= 16);
		let low_mask: u16 = if position == 0 { 0 } else { (1u16 << position) - 1 };
		let low = bits & low_mask;
		let high_count = len - position - 1;
		let high_mask: u16 = if high_count == 0 {
			0
		} else {
			// `high_count <= 15` and `position + 1 + high_count == len <= 16`,
			// so neither shift overflows.
			((1u16 << high_count) - 1) << (position + 1)
		};
		let high = (bits & high_mask) >> 1;
		low | high
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

	/// Marks slot `position` as referenced (sets its bit), then delegates to
	/// the policy. Under `ClassicLRU` the policy performs a `remove+push`
	/// that moves the item at `position` to the back; we mirror the same
	/// permutation on `ref_bits` so the just-set bit travels to slot
	/// `len-1` alongside its item, and any bits above `position` shift down
	/// by one to fill the gap.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let len = self.items.len();
		debug_assert!(position < len, "get_from_position: position {position} out of bounds (len={len})");

		// Mark slot as referenced before delegation.
		let bits = self.ref_bits.get();
		self.ref_bits.set(bits | (1u16 << position));

		let new_index = self.eviction_policy.get_from_position(&mut self.items, position);

		// Mirror the policy's items-permutation onto `ref_bits`.
		//
		// Under ClassicLRU the policy's `remove+push` moves the slot at
		// `position` to `len-1`. The bit at `position` (just set to 1) must
		// travel with it; bits at `position+1..len` must shift down by one
		// to fill the gap; the bit at the new trailing slot `len-1` is the
		// extracted bit, i.e. `1`.
		//
		// For every other policy the items do not move this way, but the
		// bits are never read for eviction, so the bookkeeping is harmless.
		debug_assert_eq!(self.items.len(), len, "policy must not change len on get");
		let bits = self.ref_bits.get();
		let collapsed = Self::collapse_at(bits, position, len);
		let last_bit: u16 = 1u16 << (len - 1);
		self.ref_bits.set(collapsed | last_bit);

		&self.items[new_index]
	}

	/// CLOCK-style eviction under `ClassicLRU`:
	///   - return the first slot whose `ref_bits` bit is `0` (untouched), or
	///   - if every slot is set, mass-clear `ref_bits` (temporal aging step)
	///     and evict slot `0` (the LRU front).
	///
	/// For every other policy, delegate to the policy exactly as the trait
	/// default does — the bitmask is not consulted.
	fn eviction_candidate_index(&self) -> Option<usize> {
		let len = self.items.len();
		if len == 0 {
			return None;
		}
		if self.eviction_policy.name() == "ClassicLRU" {
			let bits = self.ref_bits.get();
			for i in 0..len {
				if (bits >> i) & 1 == 0 {
					return Some(i);
				}
			}
			// All resident slots are touched: mass-clear (aging sweep) and
			// evict the LRU front. Next eviction can again discriminate
			// untouched-vs-touched.
			self.ref_bits.set(0);
			Some(0)
		} else {
			Some(self.eviction_policy.eviction_candidate_index(&self.items))
		}
	}

	/// Removes the item at `index` and mirrors `Vec::remove` on `ref_bits`:
	/// bits at `index+1..old_len` shift down by one; the new trailing bit at
	/// `old_len-1` is cleared (i.e. preserves the "bits beyond len are 0"
	/// invariant for the next push).
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		let old_len = self.items.len();
		debug_assert!(
			index < old_len,
			"remove_at: index {index} out of bounds (len={old_len})"
		);
		let item = self.items.remove(index);
		let bits = self.ref_bits.get();
		self.ref_bits.set(Self::collapse_at(bits, index, old_len));
		item
	}

	/// Wraps the trait default `put` with a post-push clear of the new
	/// slot's bit, so a freshly-inserted item starts out as "untouched"
	/// (the first eviction sweep can pick it before any item that has been
	/// hit at least once). For an existing-key update, falls through to
	/// `get_from_position`, which sets the bit and performs the policy's
	/// permutation — matching how the trait default handles that branch.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.items.push(item);
		let new_pos = self.items.len() - 1;
		// Clear the new slot's bit: untouched by construction.
		let bits = self.ref_bits.get();
		self.ref_bits.set(bits & !(1u16 << new_pos));
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
