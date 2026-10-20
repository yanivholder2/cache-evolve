use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Segmented (probationary / protected) bucket.
///
/// `items` holds every item of the bucket in one vector ordered as
/// `[probationary region | protected region]`. The first `prob_len` entries are
/// probationary; the remainder are protected. New misses enter the probationary
/// region; a probationary hit promotes the entry to the protected region; when
/// the protected region is full the promotion demotes its policy-chosen victim
/// back to probation. Eviction takes the victim from probation first and only
/// falls back to protected when probation is empty. The probationary victim is
/// the entry with the smallest shared frequency (ties keep the policy's pick).
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
	/// Capacity of the protected region (≈75% of `max_size`).
	prot_cap: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let max_size = settings.bucket_size;
		// Probationary ≈ 25% of slots, rounded, at least one slot.
		let prob_cap = ((max_size + 2) / 4).max(1);
		let prot_cap = max_size.saturating_sub(prob_cap);
		Self {
			items: Vec::with_capacity(max_size),
			max_size,
			prob_len: 0,
			prot_cap,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	fn protected_len(&self) -> usize {
		self.items.len() - self.prob_len
	}

	/// Absolute index of the probationary victim: the probationary entry with the
	/// smallest shared frequency as reported by `freq`. When the policy's own pick
	/// is tied for the minimum it wins; otherwise the first minimal entry is taken.
	/// Precondition: `prob_len > 0`.
	fn probation_victim_by<F: Fn(&K) -> usize>(&self, freq: &F) -> usize {
		let prob = &self.items[..self.prob_len];
		let policy_pick = self.eviction_policy.eviction_candidate_index(prob);
		let min = prob.iter().map(|it| freq(&it.key)).min().unwrap_or(0);
		if freq(&prob[policy_pick].key) == min {
			return policy_pick;
		}
		prob.iter().position(|it| freq(&it.key) == min).unwrap_or(policy_pick)
	}

	/// Absolute index of the protected victim. Precondition: protected region non-empty.
	fn protected_victim(&self) -> usize {
		self.prob_len + self.eviction_policy.eviction_candidate_index(&self.items[self.prob_len..])
	}

	/// Absolute index of the entry eviction would remove, given shared frequencies.
	/// Probation first; protected only when probation is empty.
	pub fn victim_index<F: Fn(&K) -> usize>(&self, freq: &F) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		Some(if self.prob_len > 0 {
			self.probation_victim_by(freq)
		} else {
			self.protected_victim()
		})
	}

	/// Applies the policy's access side effect within the protected region.
	/// Returns the absolute index of the touched item.
	fn touch_protected(&mut self, local: usize) -> usize {
		let mut prot = self.items.split_off(self.prob_len);
		let idx = self.eviction_policy.get_from_position(&mut prot, local);
		self.items.append(&mut prot);
		self.prob_len + idx
	}

	/// Promotes the probationary item at `pos` into the protected region,
	/// demoting the protected victim to probation if the protected region is full.
	/// Returns the absolute index of the promoted item. Requires `prot_cap > 0`.
	fn promote(&mut self, pos: usize) -> usize {
		let item = self.items.remove(pos);
		self.prob_len -= 1;
		if self.protected_len() == self.prot_cap {
			let victim = self.protected_victim();
			let demoted = self.items.remove(victim);
			self.items.insert(self.prob_len, demoted);
			self.prob_len += 1;
		}
		self.items.push(item);
		let local = self.items.len() - 1 - self.prob_len;
		self.touch_protected(local)
	}

	/// Access handling for the item at absolute `pos`. Returns its resulting absolute index.
	fn touch(&mut self, pos: usize) -> usize {
		if pos < self.prob_len {
			if self.prot_cap == 0 {
				pos
			} else {
				self.promote(pos)
			}
		} else {
			self.touch_protected(pos - self.prob_len)
		}
	}

	/// Removes the item at absolute `idx`, keeping `prob_len` consistent.
	fn remove_absolute(&mut self, idx: usize) -> Item<K, V> {
		if idx < self.prob_len {
			self.prob_len -= 1;
		}
		self.items.remove(idx)
	}

	/// Evicts one victim under the policy alone (no shared frequencies).
	fn evict_victim(&mut self) -> Option<Item<K, V>> {
		let idx = self.victim_index(&|_: &K| 0)?;
		Some(self.remove_absolute(idx))
	}

	/// Inserts `item` like `put`, but when the bucket is full and the key is absent
	/// evicts the entry at `victim`. `victim` must be an index produced by
	/// `victim_index` over the current state, so the admission decision and the
	/// removal agree. `None` falls back to the policy-only victim.
	pub fn put_with_victim(
		&mut self,
		item: Item<K, V>,
		victim: Option<usize>,
	) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let idx = self.touch(position);
			return (&self.items[idx], None);
		}
		let evicted = if self.is_full() {
			match victim {
				Some(idx) => Some(self.remove_absolute(idx)),
				None => self.evict_victim(),
			}
		} else {
			None
		};
		self.items.insert(self.prob_len, item);
		self.prob_len += 1;
		(&self.items[self.prob_len - 1], evicted)
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
		self.put_with_victim(item, None)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let idx = self.touch(position);
		&self.items[idx]
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		self.victim_index(&|_: &K| 0)
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
