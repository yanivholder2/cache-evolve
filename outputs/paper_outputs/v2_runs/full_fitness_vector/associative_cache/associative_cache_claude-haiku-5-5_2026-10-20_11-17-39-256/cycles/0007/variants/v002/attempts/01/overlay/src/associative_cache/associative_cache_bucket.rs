use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Per-slot access frequency saturates here (a 2-bit-style counter).
const FREQ_SATURATION: usize = 3;
/// Every this many insertions into a bucket, all of its frequency counters are halved.
const AGING_PERIOD: usize = 256;
/// Sentinel for an unfilled entry in the lowest-counter scan.
const NO_SLOT: usize = usize::MAX;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Count of insertions into this bucket, driving the periodic aging pass.
	insertions: usize,
	/// Two candidate victim slots recorded by the last full scan: the two lowest-counter
	/// slots that remain after that scan's victim was evicted. Compact (two u8 indices plus
	/// the valid flag as `Option`). Only a shortcut for the scan: the indices are candidates,
	/// and the choice between them uses the current counters. `None` means invalid.
	victim_pair: Option<(u8, u8)>,
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
			insertions: 0,
			victim_pair: None,
		}
	}

	/// Returns the indices of the three lowest-counter slots, ascending by counter.
	/// Ties keep the earlier index first, matching `min_by_key`'s front-of-bucket choice.
	/// Unfilled entries are `NO_SLOT`.
	fn three_lowest_slots(&self) -> [usize; 3] {
		let mut top = [NO_SLOT; 3];
		for (index, item) in self.items.iter().enumerate() {
			let counter = item.lfu_counter;
			let insert_at = (0..3).find(|&k| top[k] == NO_SLOT || counter < self.items[top[k]].lfu_counter);
			if let Some(pos) = insert_at {
				for k in (pos + 1..3).rev() {
					top[k] = top[k - 1];
				}
				top[pos] = index;
			}
		}
		top
	}

	/// Evicts the victim for a put into a full bucket.
	///
	/// If the cached pair is valid, the victim is the lower-counter of the two cached slots
	/// (current counters), and the cache is invalidated. Otherwise the bucket is scanned once,
	/// the lowest-counter slot is evicted, and the two next-lowest slots are recorded for the
	/// next full put.
	fn evict_for_insert(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			self.victim_pair = None;
			return None;
		}
		if let Some((x, y)) = self.victim_pair.take() {
			let (x, y) = (usize::from(x), usize::from(y));
			if x < self.items.len() && y < self.items.len() {
				let victim = if self.items[y].lfu_counter < self.items[x].lfu_counter {
					y
				} else {
					x
				};
				return Some(self.remove_at(victim));
			}
		}

		let top = self.three_lowest_slots();
		let victim = top[0];
		let evicted = self.remove_at(victim);
		// Indices above the removed slot shift down by one.
		let shift = |idx: usize| if idx > victim { idx - 1 } else { idx };
		self.victim_pair = match (top[1], top[2]) {
			(a, b) if a != NO_SLOT && b != NO_SLOT && shift(a) <= u8::MAX as usize && shift(b) <= u8::MAX as usize => {
				Some((shift(a) as u8, shift(b) as u8))
			}
			_ => None,
		};
		Some(evicted)
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

	/// Insert a new key (or update an existing one). Slots start with frequency 0,
	/// and the bucket ages all frequency counters every `AGING_PERIOD` insertions.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_for_insert() } else { None };
		self.items.push(item);
		self.insertions += 1;
		if self.insertions % AGING_PERIOD == 0 {
			for slot in self.items.iter_mut() {
				slot.lfu_counter >>= 1;
			}
		}
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
	}

	/// Victim selection: the slot with the smallest frequency counter. Ties go to
	/// the lowest index (`min_by_key` returns the first minimum).
	fn eviction_candidate_index(&self) -> Option<usize> {
		self.items
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| item.lfu_counter)
			.map(|(index, _)| index)
	}

	/// Promotion is unchanged (delegated to the policy); the hit additionally bumps
	/// the slot's saturating frequency counter. Counter increments do not invalidate
	/// the cached victim pair: it is only a candidate shortlist.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		let item = &mut self.items[index];
		item.lfu_counter = (item.lfu_counter + 1).min(FREQ_SATURATION);
		&self.items[index]
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
