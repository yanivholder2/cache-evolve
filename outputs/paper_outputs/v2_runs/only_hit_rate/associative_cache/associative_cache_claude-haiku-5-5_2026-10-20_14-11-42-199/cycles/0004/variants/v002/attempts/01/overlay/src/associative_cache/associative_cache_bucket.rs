use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Upper bound on the number of probationary slots visited by one victim selection.
/// A probationary region of at most this many slots is scanned exhaustively.
const PROBATION_SCAN_LIMIT: usize = 8;

/// Segmented (probationary / protected) bucket.
///
/// `items` holds every item of the bucket in one vector ordered as
/// `[probationary region | protected region]`. The first `prob_len` entries are
/// probationary; the remainder are protected. New misses enter the probationary
/// region; a probationary hit promotes the entry to the protected region; when
/// the protected region is full the promotion demotes its policy-chosen victim
/// back to probation. Eviction takes the victim from probation first and only
/// falls back to protected when probation is empty. The configured eviction
/// policy supplies the recency/frequency ordering within each region.
///
/// Probationary victim selection is bounded: it visits at most
/// `PROBATION_SCAN_LIMIT` probationary slots, a window that starts at a rotating
/// per-bucket cursor which advances by the number of slots visited on each
/// eviction.
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
	/// Rotating start position of the bounded probationary scan window.
	probation_cursor: usize,
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
			probation_cursor: 0,
			prot_cap,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	fn protected_len(&self) -> usize {
		self.items.len() - self.prob_len
	}

	/// Bounded probationary scan window as `(start, len)`, absolute indices into `items`.
	/// Covers the whole probationary region when it has at most `PROBATION_SCAN_LIMIT`
	/// slots; otherwise a contiguous window of `PROBATION_SCAN_LIMIT` slots starting at
	/// the rotating cursor (clamped so the window stays inside the region).
	/// Precondition: `prob_len > 0`.
	fn probation_window(&self) -> (usize, usize) {
		let n = self.prob_len;
		if n <= PROBATION_SCAN_LIMIT {
			return (0, n);
		}
		let start = self.probation_cursor % n;
		let start = if start + PROBATION_SCAN_LIMIT > n {
			n - PROBATION_SCAN_LIMIT
		} else {
			start
		};
		(start, PROBATION_SCAN_LIMIT)
	}

	/// Absolute index of the probationary victim, chosen among the scan window.
	/// Precondition: `prob_len > 0`.
	fn probation_victim(&self) -> usize {
		let (start, len) = self.probation_window();
		start + self.eviction_policy.eviction_candidate_index(&self.items[start..start + len])
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

	/// Evicts one victim: probation first, protected only if probation is empty.
	fn evict_victim(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}
		let idx = if self.prob_len > 0 {
			let victim = self.probation_victim();
			let (_, visited) = self.probation_window();
			self.probation_cursor = (self.probation_cursor + visited) % self.prob_len;
			victim
		} else {
			self.protected_victim()
		};
		if idx < self.prob_len {
			self.prob_len -= 1;
		}
		Some(self.items.remove(idx))
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
