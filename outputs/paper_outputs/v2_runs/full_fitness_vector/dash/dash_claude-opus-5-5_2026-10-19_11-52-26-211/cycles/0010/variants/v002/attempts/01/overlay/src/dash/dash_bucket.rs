use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::traits::eviction_policy::EvictionPolicy;
use crate::shared::utils::hash;
use log::debug;
use std::{
	fmt::{Debug, Display, Formatter},
	hash::Hash,
};

/// Largest bucket capacity the inline tag array covers. Buckets with a larger
/// `max_size` keep working but never enable tags (plain key scan).
pub const MAX_SLOTS: usize = 64;

/// One-byte key tag taken from the top byte of the key hash. Segment / bucket /
/// stash indexing uses `hash % n` for small `n`, so the top byte is effectively
/// independent of the index a key landed in.
#[inline(always)]
pub fn tag_of_hash(h: usize) -> u8 {
	((h as u64) >> 56) as u8
}

/// Computes the tag of `key` from scratch (one hash). Used only where no hash is
/// already in hand (fixture inserts, tag rebuilds, consistency checks).
#[inline]
pub fn key_tag<K: Hash + Debug>(key: &K) -> u8 {
	tag_of_hash(hash(key))
}

/// A bucket header with an inline per-slot tag array.
///
/// `tags[j]` mirrors the key of `items[j]` while `tag_ok` is set. Lookups compare
/// `tags[..len]` against the probe tag first and only load `items[j]` (the heap
/// buffer) for slots whose tag matches. Tags only filter: a tag match always
/// falls through to the full key compare.
///
/// `tag_ok` is cleared whenever the raw items vector is handed out mutably
/// (`get_items_mut`), since the caller may then mutate it behind the tag array;
/// lookups then fall back to a plain key scan, and the next insert rebuilds tags.
#[derive(Debug)]
#[repr(C)]
pub struct DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: u32,
	tag_ok: bool,
	tags: [u8; MAX_SLOTS],
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(max_size: usize, eviction_policy: Box<dyn EvictionPolicy<K, V>>) -> Self {
		let max_size_u32 = u32::try_from(max_size).expect("DashBucket::new: max_size exceeds u32");
		DashBucket {
			items: Vec::with_capacity(max_size),
			max_size: max_size_u32,
			tag_ok: max_size <= MAX_SLOTS,
			tags: [0u8; MAX_SLOTS],
			eviction_policy,
		}
	}
}

/// Boxed constructor, kept for any caller that wants a `Box<dyn Bucket>`.
#[allow(dead_code)]
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

/// By-value constructor: lets `SegmentCore` store buckets contiguously in a
/// `Vec<DashBucket<K, V>>` with statically dispatched method calls.
#[inline]
pub fn new_bucket_concrete<K, V>(
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
) -> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	DashBucket::new(max_size, eviction_policy)
}

// ── Inherent, statically dispatched hot-path methods ─────────────────────────
// Inherent methods take precedence over the trait's, so `SegmentCore` call sites
// (`buckets[i].get_position(..)` etc.) resolve here with no vtable and no import.
// Each mirrors the `Bucket` trait's default semantics exactly, while keeping the
// tag array in sync with the items vector.
#[allow(dead_code)]
impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	#[inline]
	pub fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	/// Hands out the raw items vector. The caller may reorder/insert/remove behind
	/// the tag array, so tags are invalidated (lookups fall back to a plain scan
	/// until the next insert rebuilds them).
	#[inline]
	pub fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		self.tag_ok = false;
		&mut self.items
	}

	#[inline]
	pub fn get_max_size(&self) -> usize {
		self.max_size as usize
	}

	#[inline]
	pub fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	/// Whether the tag array currently mirrors the items (test/diagnostic helper).
	#[inline]
	pub fn tags_enabled(&self) -> bool {
		self.tag_ok
	}

	/// Re-establishes `tags` from the items when they were invalidated (cold path:
	/// only reached after a raw `get_items_mut` mutation).
	#[inline]
	fn ensure_tags(&mut self) {
		if !self.tag_ok {
			self.rebuild_tags();
		}
	}

	#[cold]
	#[inline(never)]
	fn rebuild_tags(&mut self) {
		let len = self.items.len();
		if self.max_size as usize > MAX_SLOTS || len > MAX_SLOTS {
			return;
		}
		for j in 0..len {
			self.tags[j] = key_tag(&self.items[j].key);
		}
		self.tag_ok = true;
	}

	/// `true` if tags are disabled or exactly mirror the items' keys.
	pub fn tags_consistent(&self) -> bool {
		!self.tag_ok
			|| (self.items.len() <= MAX_SLOTS
				&& self
					.items
					.iter()
					.enumerate()
					.all(|(j, it)| self.tags[j] == key_tag(&it.key)))
	}

	/// Plain key-equality scan over the items slice (no tag in hand).
	#[inline]
	pub fn get_position(&self, key: &K) -> Option<usize> {
		self.items.iter().position(|d| d.key == *key)
	}

	/// Tag-filtered lookup: scans the inline `tags[..len]` and only dereferences the
	/// item buffer for slots whose tag equals `tag`. If no tag matches, returns
	/// `None` without touching the item buffer.
	#[inline]
	pub fn get_position_tagged(&self, key: &K, tag: u8) -> Option<usize> {
		if self.tag_ok {
			let len = self.items.len();
			let tags = &self.tags[..len];
			for (j, &t) in tags.iter().enumerate() {
				if t == tag && self.items[j].key == *key {
					return Some(j);
				}
			}
			None
		} else {
			self.get_position(key)
		}
	}

	#[inline]
	pub fn size(&self) -> usize {
		self.items.len()
	}

	#[inline]
	pub fn is_full(&self) -> bool {
		self.items.len() == self.max_size as usize
	}

	/// Touches the item per the eviction policy (recency update) and returns it.
	///
	/// Every built-in policy moves the touched item from `position` to the returned
	/// index with the items in between shifting by one (ClassicLRU: remove + push;
	/// Swap: adjacent swap, which is the same rotation), so the tag array is rotated
	/// the same way.
	#[inline]
	pub fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		if self.tag_ok && index != position {
			if index > position {
				self.tags[position..=index].rotate_left(1);
			} else {
				self.tags[index..=position].rotate_right(1);
			}
		}
		debug_assert!(self.tags_consistent(), "tag array out of sync after get_from_position");
		&self.items[index]
	}

	#[inline]
	pub fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		let len = self.items.len();
		let removed = self.items.remove(index);
		if self.tag_ok {
			self.tags.copy_within(index + 1..len, index);
		}
		removed
	}

	#[inline]
	pub fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		Some(self.eviction_policy.eviction_candidate_index(&self.items))
	}

	#[inline]
	pub fn promotion_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		Some(self.eviction_policy.promotion_candidate_index(&self.items))
	}

	/// `put_at` with the item's tag supplied by the caller (from a hash it already
	/// computed). Same semantics as the trait default `put_at`: updates in place if
	/// the key is present, else evicts per policy when full, then inserts at `pos`.
	#[inline]
	pub fn put_at_tagged(
		&mut self,
		item: Item<K, V>,
		tag: u8,
		pos: InsertPosition,
	) -> (&Item<K, V>, Option<Item<K, V>>) {
		debug_assert_eq!(tag, key_tag(&item.key), "put_at_tagged: caller tag mismatch");
		if let Some(position) = self.get_position_tagged(&item.key, tag) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		self.ensure_tags();
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		let len = self.items.len();
		if self.tag_ok && len >= MAX_SLOTS {
			// Cannot happen while len <= max_size <= MAX_SLOTS; stay correct regardless.
			self.tag_ok = false;
		}
		let pushed_item = match pos {
			InsertPosition::Back => {
				self.items.push(item);
				if self.tag_ok {
					self.tags[len] = tag;
				}
				self.items.last().expect("bucket invariant: just pushed")
			}
			InsertPosition::Front => {
				self.items.insert(0, item);
				if self.tag_ok {
					self.tags.copy_within(0..len, 1);
					self.tags[0] = tag;
				}
				self.items.first().expect("bucket invariant: just inserted")
			}
		};
		(pushed_item, evicted_item)
	}

	/// Untagged `put_at`: no hash in hand, so the tag is computed here once.
	#[inline]
	pub fn put_at(&mut self, item: Item<K, V>, pos: InsertPosition) -> (&Item<K, V>, Option<Item<K, V>>) {
		let tag = key_tag(&item.key);
		self.put_at_tagged(item, tag, pos)
	}

	#[inline]
	pub fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		self.put_at(item, InsertPosition::Back)
	}

	#[inline]
	pub fn evict_item(&mut self) -> Option<Item<K, V>> {
		let idx = self.eviction_candidate_index()?;
		let evicted = self.remove_at(idx);
		debug!(
			"Bucket.evict_item: policy={} evicted_key={:?}",
			self.eviction_policy.name(),
			evicted.key
		);
		Some(evicted)
	}

	#[inline]
	pub fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let position = self.get_position(key)?;
		Some(self.get_from_position(position))
	}

	#[inline]
	pub fn remove(&mut self, key: &K) {
		self.items.retain(|item| item.key != *key);
		// `retain` can drop arbitrary slots; rebuild tags on the next insert.
		self.tag_ok = false;
	}
}

impl<K, V> Bucket<K, V> for DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	#[inline]
	fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	#[inline]
	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		DashBucket::get_items_mut(self)
	}

	#[inline]
	fn get_max_size(&self) -> usize {
		self.max_size as usize
	}

	#[inline]
	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	#[inline]
	fn put_at(&mut self, item: Item<K, V>, pos: InsertPosition) -> (&Item<K, V>, Option<Item<K, V>>) {
		DashBucket::put_at(self, item, pos)
	}

	#[inline]
	fn remove(&mut self, key: &K) {
		DashBucket::remove(self, key)
	}

	#[inline]
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		DashBucket::get_from_position(self, position)
	}

	#[inline]
	fn get_position(&self, key: &K) -> Option<usize> {
		DashBucket::get_position(self, key)
	}

	#[inline]
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		DashBucket::evict_item(self)
	}

	#[inline]
	fn is_full(&self) -> bool {
		DashBucket::is_full(self)
	}

	#[inline]
	fn size(&self) -> usize {
		self.items.len()
	}

	#[inline]
	fn eviction_candidate_index(&self) -> Option<usize> {
		DashBucket::eviction_candidate_index(self)
	}

	#[inline]
	fn promotion_candidate_index(&self) -> Option<usize> {
		DashBucket::promotion_candidate_index(self)
	}

	#[inline]
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		DashBucket::remove_at(self, index)
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

	fn lcg(state: &mut u64) -> u64 {
		*state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
		*state >> 33
	}

	/// Random put/get/evict sequences under every policy keep the tag array exactly
	/// in sync, and tagged lookup agrees with the plain scan.
	#[test]
	fn tags_stay_in_sync_under_random_ops() {
		for policy in EvictionPolicyKind::all() {
			for size in [1usize, 2, 3, 5, 8] {
				let mut b: DashBucket<i64, i64> = DashBucket::new(size, policy.build());
				let mut st = 0xDEAD_BEEF_u64 ^ size as u64;
				for _ in 0..2000 {
					let k = (lcg(&mut st) % 24) as i64;
					let tag = key_tag(&k);
					let plain = b.get_position(&k);
					assert_eq!(b.get_position_tagged(&k, tag), plain);
					match lcg(&mut st) % 4 {
						0 | 1 => {
							if let Some(p) = plain {
								b.get_from_position(p);
							} else {
								let pos = if lcg(&mut st) % 2 == 0 {
									InsertPosition::Front
								} else {
									InsertPosition::Back
								};
								b.put_at_tagged(Item::new(k, k), tag, pos);
							}
						}
						2 => {
							if let Some(p) = plain {
								b.remove_at(p);
							}
						}
						_ => {
							b.evict_item();
						}
					}
					assert!(b.tags_enabled());
					assert!(b.tags_consistent(), "policy={policy:?} size={size}");
				}
			}
		}
	}

	#[test]
	fn raw_mutation_disables_then_rebuilds_tags() {
		let mut b: DashBucket<i64, i64> = DashBucket::new(4, EvictionPolicyKind::ClassicLRU.build());
		b.put(Item::new(1, 1));
		b.get_items_mut().push(Item::new(2, 2));
		assert!(!b.tags_enabled());
		assert_eq!(b.get_position_tagged(&2, key_tag(&2)), Some(1));
		b.put(Item::new(3, 3));
		assert!(b.tags_enabled());
		assert!(b.tags_consistent());
		assert_eq!(b.get_position_tagged(&2, key_tag(&2)), Some(1));
	}

	#[test]
	fn oversized_bucket_falls_back_to_plain_scan() {
		let mut b: DashBucket<i64, i64> = DashBucket::new(MAX_SLOTS + 1, EvictionPolicyKind::ClassicLRU.build());
		for k in 0..(MAX_SLOTS as i64 + 1) {
			b.put(Item::new(k, k));
		}
		assert!(!b.tags_enabled());
		for k in 0..(MAX_SLOTS as i64 + 1) {
			assert_eq!(b.get_position_tagged(&k, key_tag(&k)), Some(k as usize));
		}
	}
}
