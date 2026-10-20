use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Segmented (probationary / protected) bucket with second-hit promotion.
///
/// `items` holds every item of the bucket in one vector ordered as
/// `[probationary region | protected region]`. The first `prob_len` entries are
/// probationary; the remainder are protected. New misses enter the probationary
/// region. A probationary entry hit once is only marked as having been hit; a
/// second hit on a marked entry promotes it to the protected region. When the
/// protected region is full the promotion demotes its policy-chosen victim back
/// to probation, keeping its hit mark. Eviction takes the victim from probation
/// first and only falls back to protected when probation is empty. The configured
/// eviction policy supplies the recency/frequency ordering within each region.
///
/// The hit mark is bucket-local state: `hit_once` holds the keys of the entries
/// that have been hit while probationary (or demoted with the mark kept). It is
/// kept outside `Item` so the shared item layout and the policies are untouched.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	/// Number of entries at the front of `items` that belong to the probationary region.
	prob_len: usize,
	/// Capacity of the protected region (half of `max_size`).
	prot_cap: usize,
	/// Keys of resident entries carrying the "hit once" mark. Invariant: every key
	/// here is resident in `items`, and no key appears twice.
	hit_once: Vec<K>,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let max_size = settings.bucket_size;
		// Protected region capped at half the slots; the rest is probationary.
		let prot_cap = max_size / 2;
		Self {
			items: Vec::with_capacity(max_size),
			max_size,
			prob_len: 0,
			prot_cap,
			hit_once: Vec::with_capacity(max_size),
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	fn protected_len(&self) -> usize {
		self.items.len() - self.prob_len
	}

	/// Absolute index of the probationary victim. Precondition: `prob_len > 0`.
	fn probation_victim(&self) -> usize {
		self.eviction_policy
			.eviction_candidate_index(&self.items[..self.prob_len])
	}

	/// Absolute index of the protected victim. Precondition: protected region non-empty.
	fn protected_victim(&self) -> usize {
		self.prob_len + self.eviction_policy.eviction_candidate_index(&self.items[self.prob_len..])
	}

	/// Applies the policy's access side effect within the protected region.
	/// Returns the absolute index of the touched item.
	fn touch_protected(&mut self, local: usize) -> usize {
		let mut prot = self.items.split_off(self.prob_len);
		let idx = self.eviction_policy.get_from_position(&mut prot, local);
		self.items.append(&mut prot);
		self.prob_len + idx
	}

	/// Removes and returns the hit mark for `key`, if present.
	fn take_hit_mark(&mut self, key: &K) -> bool {
		match self.hit_once.iter().position(|k| k == key) {
			Some(i) => {
				self.hit_once.swap_remove(i);
				true
			}
			None => false,
		}
	}

	/// Promotes the probationary item at `pos` into the protected region,
	/// demoting the protected victim to probation (keeping its hit mark) if the
	/// protected region is full. Returns the absolute index of the promoted item.
	/// Requires `prot_cap > 0`.
	fn promote(&mut self, pos: usize) -> usize {
		let item = self.items.remove(pos);
		self.prob_len -= 1;
		if self.protected_len() == self.prot_cap {
			let victim = self.protected_victim();
			let demoted = self.items.remove(victim);
			self.hit_once.push(demoted.key);
			self.items.insert(self.prob_len, demoted);
			self.prob_len += 1;
		}
		self.items.push(item);
		let local = self.items.len() - 1 - self.prob_len;
		self.touch_protected(local)
	}

	/// Access handling for the item at absolute `pos`. Returns its resulting absolute index.
	///
	/// A probationary hit on an unmarked entry only sets its hit mark; a hit on an
	/// already-marked probationary entry promotes it.
	fn touch(&mut self, pos: usize) -> usize {
		if pos < self.prob_len {
			if self.prot_cap == 0 {
				return pos;
			}
			let key = self.items[pos].key;
			if self.take_hit_mark(&key) {
				self.promote(pos)
			} else {
				self.hit_once.push(key);
				pos
			}
		} else {
			self.touch_protected(pos - self.prob_len)
		}
	}

	/// Evicts one victim: probation first, protected only if probation is empty.
	fn evict_victim(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}
		let idx = if self.prob_len > 0 {
			self.probation_victim()
		} else {
			self.protected_victim()
		};
		if idx < self.prob_len {
			self.prob_len -= 1;
		}
		let removed = self.items.remove(idx);
		self.take_hit_mark(&removed.key);
		Some(removed)
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
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let idx = self.touch(position);
			return (&self.items[idx], None);
		}
		let evicted = if self.is_full() { self.evict_victim() } else { None };
		self.items.insert(self.prob_len, item);
		self.prob_len += 1;
		(&self.items[self.prob_len - 1], evicted)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let idx = self.touch(position);
		&self.items[idx]
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		Some(if self.prob_len > 0 {
			self.probation_victim()
		} else {
			self.protected_victim()
		})
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		self.evict_victim()
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
