use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

const P_INIT: usize = 16;
const P_MIN: usize = 4;
const P_MAX: usize = 32;
const P_SCALE: usize = 64;
const GHOST_CAP: usize = 8;

/// 2Q-style bucket with an adaptive probation segment.
///
/// `items` layout: `[0..probation_len)` is probation (oldest first),
/// `[probation_len..)` is protected (LRU first, MRU last).
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	probation_len: usize,
	p: usize,
	ghost: Vec<K>,
	ghost_head: usize,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			probation_len: 0,
			p: P_INIT,
			ghost: Vec::with_capacity(GHOST_CAP),
			ghost_head: 0,
		}
	}

	fn target(&self) -> usize {
		(self.p * self.max_size / P_SCALE).max(1)
	}

	fn victim_index(&self) -> usize {
		if self.probation_len > 0 && (self.probation_len > self.target() || self.probation_len == self.items.len()) {
			0
		} else if self.probation_len < self.items.len() {
			self.probation_len
		} else {
			0
		}
	}

	fn record_ghost(&mut self, key: K) {
		if self.ghost.len() < GHOST_CAP {
			self.ghost.push(key);
		} else {
			self.ghost[self.ghost_head] = key;
			self.ghost_head = (self.ghost_head + 1) % GHOST_CAP;
		}
	}

	fn check_ghost(&mut self, key: &K) {
		if let Some(i) = self.ghost.iter().position(|k| k == key) {
			// Keep ring order stable: overwrite slot with a non-matching duplicate is
			// unnecessary; simply swap-remove and reset the head if needed.
			self.ghost.remove(i);
			if self.ghost_head >= self.ghost.len() {
				self.ghost_head = 0;
			}
			self.p = (self.p + 1).min(P_MAX);
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
		if position < self.probation_len {
			// second hit: promote to protected MRU
			self.probation_len -= 1;
		} else {
			let prot_len = self.items.len() - self.probation_len;
			let rel = position - self.probation_len;
			if rel < prot_len.div_ceil(4) {
				self.p = self.p.saturating_sub(1).max(P_MIN);
			}
		}
		let item = self.items.remove(position);
		self.items.push(item);
		self.items.last().expect("just pushed")
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed = self.get_from_position(position);
			return (pushed, None);
		}
		self.check_ghost(&item.key);
		let evicted = if self.is_full() { self.evict_item() } else { None };
		let at = self.probation_len;
		self.items.insert(at, item);
		self.probation_len += 1;
		(&self.items[at], evicted)
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			None
		} else {
			Some(self.victim_index())
		}
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		let idx = self.eviction_candidate_index()?;
		let from_probation = idx < self.probation_len;
		let evicted = self.remove_at(idx);
		if from_probation {
			self.record_ghost(evicted.key);
		}
		Some(evicted)
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		if index < self.probation_len {
			self.probation_len -= 1;
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
		for item in &self.items {
			writeln!(f, "    {}", item)?;
		}
		Ok(())
	}
}
