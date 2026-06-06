use crate::shared::item::Item;
use crate::shared::traits::bucket::{lru_get_from_position, Bucket, InsertPosition};
use crate::shared::traits::eviction_policy::EvictionPolicy;
use std::{
	fmt::{Debug, Display, Formatter},
	hash::Hash,
};

#[derive(Debug)]
pub struct DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// H03 promotion-heat, kept strictly index-aligned with `items`. The
	/// frequency signal that steers a target resident into the protected probing
	/// tier used to live in a segment-owned `HashMap<K, u8>` side table (H01),
	/// probed by hashing the key on every target hit. It now lives here, beside
	/// the items it describes: `heat[i]` is the heat of `items[i]`. Every
	/// mutation that reorders, inserts into, or removes from `items` applies the
	/// identical transform to `heat`, so the parallel arrays never drift. This is
	/// bucket container state, not per-`Item` layout, so `Item` stays at the
	/// 16-byte floor while the hashed probe on the hot path is gone.
	heat: Vec<u8>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// H01 (port of cycle/0004/v002): captured once at construction from
	/// `eviction_policy.name() == "ClassicLRU"`. ClassicLRU is the policy on the
	/// evaluated path; this flag lets the hot path call inlined free functions
	/// instead of dispatching through the `Box<dyn EvictionPolicy>` vtable, while
	/// any other policy still routes through the trait object unchanged.
	is_classic_lru: bool,
}

impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(max_size: usize, eviction_policy: Box<dyn EvictionPolicy<K, V>>) -> Self {
		let is_classic_lru = eviction_policy.name() == "ClassicLRU";
		DashBucket {
			items: Vec::with_capacity(max_size),
			heat: Vec::with_capacity(max_size),
			max_size,
			eviction_policy,
			is_classic_lru,
		}
	}

	/// Restores `heat` to the same length as `items`, padding with `0`. The
	/// evaluated and invariant-gate paths drive the bucket solely through the
	/// trait methods overridden below, which keep the arrays aligned. This guard
	/// only covers the negative-fixture paths that push directly into
	/// `get_items_mut()` (bypassing `put_at`), so a later `heat_at`/`bump_heat_at`
	/// stays in bounds instead of panicking.
	#[inline]
	fn realign_heat(&mut self) {
		if self.heat.len() != self.items.len() {
			self.heat.resize(self.items.len(), 0);
		}
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

impl<K, V> Bucket<K, V> for DashBucket<K, V>
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

	fn is_classic_lru(&self) -> bool {
		self.is_classic_lru
	}

	// ── H03 promotion-heat accessors ─────────────────────────────────────────
	fn heat_at(&self, pos: usize) -> u8 {
		self.heat.get(pos).copied().unwrap_or(0)
	}

	fn bump_heat_at(&mut self, pos: usize) -> u8 {
		// Keep `heat` at least as long as `items` so an in-range item index is a
		// valid heat index even after a direct-push fixture path.
		if self.heat.len() < self.items.len() {
			self.realign_heat();
		}
		match self.heat.get_mut(pos) {
			Some(h) => {
				*h = h.saturating_add(1);
				*h
			}
			None => 0,
		}
	}

	// ── Mutation overrides that keep `heat` index-aligned with `items` ───────
	// `put_at`, `remove`, `remove_at` and `get_from_position` are the only paths
	// through which the evaluated and invariant-gate workloads mutate the bucket.
	// Each applies the identical index transform to `heat` that it applies to
	// `items`, so the two arrays never drift on those paths.

	fn put_at(&mut self, item: Item<K, V>, pos: InsertPosition) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			// Update-in-place: no length change; `get_from_position` carries the
			// heat byte along with the item when it reorders.
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		match pos {
			InsertPosition::Back => {
				self.items.push(item);
				self.heat.push(0);
				(self.items.last().expect("bucket invariant: just pushed"), evicted_item)
			}
			InsertPosition::Front => {
				self.items.insert(0, item);
				self.heat.insert(0, 0);
				(self.items.first().expect("bucket invariant: just inserted"), evicted_item)
			}
		}
	}

	fn remove(&mut self, key: &K) {
		if let Some(pos) = self.items.iter().position(|it| it.key == *key) {
			self.items.remove(pos);
			if pos < self.heat.len() {
				self.heat.remove(pos);
			}
		}
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		if index < self.heat.len() {
			self.heat.remove(index);
		}
		self.items.remove(index)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		// H01: devirtualized ClassicLRU hot path. When the bucket carries
		// ClassicLRU, call the inlined free function directly (bit-for-bit
		// identical to `ClassicLRU::get_from_position`) and skip the vtable.
		// All other policies fall back to the trait object.
		let index = if self.is_classic_lru {
			// H03: mirror the ClassicLRU remove+push on `heat` so the heat byte
			// rides along with its item to the MRU end and the arrays stay aligned.
			if position < self.heat.len() {
				let h = self.heat.remove(position);
				self.heat.push(h);
			}
			lru_get_from_position(&mut self.items, position)
		} else {
			self.eviction_policy.get_from_position(&mut self.items, position)
		};
		&self.items[index]
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

	fn lru_bucket(max_size: usize) -> DashBucket<i64, i64> {
		DashBucket::new(max_size, EvictionPolicyKind::ClassicLRU.build())
	}

	#[test]
	fn put_back_seeds_zero_heat() {
		let mut b = lru_bucket(4);
		b.put(Item::new(1, 10));
		assert_eq!(b.heat_at(0), 0);
	}

	#[test]
	fn bump_then_read_back_heat() {
		let mut b = lru_bucket(4);
		b.put(Item::new(1, 10));
		assert_eq!(b.bump_heat_at(0), 1);
		assert_eq!(b.bump_heat_at(0), 2);
		assert_eq!(b.heat_at(0), 2);
	}

	/// The heat byte must travel with its item when a ClassicLRU hit reorders the
	/// bucket: after bumping key 1's heat, touching it moves it to the back and
	/// its heat must follow.
	#[test]
	fn heat_follows_item_through_lru_reorder() {
		let mut b = lru_bucket(4);
		b.put(Item::new(1, 10)); // idx 0
		b.put(Item::new(2, 20)); // idx 1
		b.put(Item::new(3, 30)); // idx 2
		b.bump_heat_at(0); // key 1 heat = 1
		// Touch key 1 (idx 0): ClassicLRU moves it to the back.
		b.get_from_position(0);
		let pos1 = b.get_position(&1).expect("key 1 still present");
		assert_eq!(pos1, 2, "key 1 moved to MRU end");
		assert_eq!(b.heat_at(pos1), 1, "key 1 kept its heat across the reorder");
		// The other items remain cold and aligned.
		let pos2 = b.get_position(&2).unwrap();
		let pos3 = b.get_position(&3).unwrap();
		assert_eq!(b.heat_at(pos2), 0);
		assert_eq!(b.heat_at(pos3), 0);
	}

	/// Removing an item drops its heat slot and keeps the remaining heats aligned.
	#[test]
	fn remove_keeps_heat_aligned() {
		let mut b = lru_bucket(4);
		b.put(Item::new(1, 10));
		b.put(Item::new(2, 20));
		b.put(Item::new(3, 30));
		b.bump_heat_at(2); // key 3 heat = 1
		b.remove(&1); // drop front; key 3 shifts from idx 2 -> 1
		let pos3 = b.get_position(&3).unwrap();
		assert_eq!(b.heat_at(pos3), 1, "key 3 kept its heat after a front removal");
		let pos2 = b.get_position(&2).unwrap();
		assert_eq!(b.heat_at(pos2), 0);
	}

	/// Eviction (full ClassicLRU put) drops the victim's heat slot; the new item
	/// enters with a fresh zero heat.
	#[test]
	fn eviction_seeds_fresh_heat_for_new_item() {
		let mut b = lru_bucket(2);
		b.put(Item::new(1, 10));
		b.put(Item::new(2, 20));
		b.bump_heat_at(0); // key 1 (LRU victim) heat = 1
		let (_, evicted) = b.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 1, "ClassicLRU evicts the front");
		let pos3 = b.get_position(&3).unwrap();
		assert_eq!(b.heat_at(pos3), 0, "re-inserted slot starts cold");
		// Arrays stay aligned: every live item has a heat byte.
		assert_eq!(b.get_items().len(), 2);
		assert_eq!(b.heat_at(b.get_position(&2).unwrap()), 0);
	}
}
