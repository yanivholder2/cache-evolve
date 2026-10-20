mod dash_segment;
mod segment;

pub use dash_segment::DashSegment;
pub use segment::Segment;

use super::dash_settings::DashSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// Put destination for an item, resolved from its key.
#[derive(Debug, Clone, Copy)]
pub enum Slot {
	Stash,
	Target,
	Probing,
}

// ── Flat-slice bucket helpers ──────────────────────────────────────────────
//
// A bucket is a `cap`-long region of a flat item array plus a `u8` length; its live
// items are `region[..len]`. These free functions reproduce the `Bucket` default
// methods and each `EvictionPolicy` exactly (same positions, same recency move on hit,
// same Front/Back meaning, same victim/hottest index), statically dispatched by a match
// on the `Copy` policy selector instead of a `Box<dyn EvictionPolicy>` vtable call.

/// Position of `key` among the live items `s`, if present.
#[inline]
pub(crate) fn find_in<K, V>(s: &[Item<K, V>], key: &K) -> Option<usize>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	s.iter().position(|it| it.key == *key)
}

/// Applies the policy's on-hit update to the item at `pos` of the live items `s`
/// (`EvictionPolicy::get_from_position`) and returns the item's new index.
#[inline]
pub(crate) fn touch<K, V>(policy: EvictionPolicyKind, s: &mut [Item<K, V>], pos: usize) -> usize
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	match policy {
		EvictionPolicyKind::ClassicLRU => {
			s[pos..].rotate_left(1);
			s.len() - 1
		}
		EvictionPolicyKind::TimestampLRU => {
			s[pos].timestamp = std::time::Instant::now();
			pos
		}
		EvictionPolicyKind::Lifo | EvictionPolicyKind::Fifo => pos,
		EvictionPolicyKind::Lfu => {
			s[pos].lfu_counter += 1;
			pos
		}
		EvictionPolicyKind::Swap => {
			if pos > 0 {
				s.swap(pos, pos - 1);
				pos - 1
			} else {
				pos
			}
		}
	}
}

/// Eviction candidate among the non-empty live items `s` (`eviction_candidate_index`).
#[inline]
pub(crate) fn victim<K, V>(policy: EvictionPolicyKind, s: &[Item<K, V>]) -> usize
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	match policy {
		EvictionPolicyKind::ClassicLRU | EvictionPolicyKind::Fifo | EvictionPolicyKind::Swap => 0,
		EvictionPolicyKind::Lifo => s.len() - 1,
		EvictionPolicyKind::TimestampLRU => {
			s.iter()
				.enumerate()
				.min_by_key(|(_, it)| it.timestamp)
				.expect("victim precondition: non-empty")
				.0
		}
		EvictionPolicyKind::Lfu => {
			s.iter()
				.enumerate()
				.min_by_key(|(_, it)| it.lfu_counter)
				.expect("victim precondition: non-empty")
				.0
		}
	}
}

/// Promotion ("hottest") candidate among the non-empty live items `s`
/// (`promotion_candidate_index`).
#[inline]
pub(crate) fn hottest<K, V>(policy: EvictionPolicyKind, s: &[Item<K, V>]) -> usize
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	match policy {
		EvictionPolicyKind::ClassicLRU | EvictionPolicyKind::Fifo | EvictionPolicyKind::Swap => s.len() - 1,
		EvictionPolicyKind::Lifo => 0,
		EvictionPolicyKind::TimestampLRU => {
			s.iter()
				.enumerate()
				.max_by_key(|(_, it)| it.timestamp)
				.expect("hottest precondition: non-empty")
				.0
		}
		EvictionPolicyKind::Lfu => {
			s.iter()
				.enumerate()
				.max_by_key(|(_, it)| it.lfu_counter)
				.expect("hottest precondition: non-empty")
				.0
		}
	}
}

/// Removes and returns the item at `pos` of a bucket `region` with live length `*len`,
/// shifting later items left (`Vec::remove` order).
#[inline]
pub(crate) fn remove_at<K, V>(region: &mut [Item<K, V>], len: &mut u8, pos: usize) -> Item<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	let l = *len as usize;
	debug_assert!(pos < l, "remove_at: index {pos} out of bounds (len={l})");
	let removed = region[pos].clone();
	region[pos..l].rotate_left(1);
	*len -= 1;
	removed
}

/// Inserts `item` at `pos` (Front = index 0, Back = end) into a bucket `region`
/// (`cap`-long) with live length `*len`, evicting the policy victim first if full.
/// Returns `(inserted_index, evicted)`.
#[inline]
pub(crate) fn insert_at<K, V>(
	policy: EvictionPolicyKind,
	region: &mut [Item<K, V>],
	len: &mut u8,
	item: Item<K, V>,
	pos: InsertPosition,
) -> (usize, Option<Item<K, V>>)
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	let evicted = if *len as usize == region.len() && *len > 0 {
		let v = victim(policy, &region[..*len as usize]);
		Some(remove_at(region, len, v))
	} else {
		None
	};
	let l = *len as usize;
	region[l] = item;
	*len += 1;
	match pos {
		InsertPosition::Back => (l, evicted),
		InsertPosition::Front => {
			region[..=l].rotate_right(1);
			(0, evicted)
		}
	}
}

/// One tier of buckets stored inline in a single flat item array: bucket `i` owns
/// `items[i*cap .. i*cap + lens[i]]`. The array is allocated once (capacity reserved in
/// the constructor) and filled with placeholder copies on the first insert, since a
/// generic `Item<K, V>` cannot be fabricated without a key; slots past `lens[i]` are
/// never read.
pub(crate) struct FlatBuckets<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	pub(crate) items: Vec<Item<K, V>>,
	pub(crate) lens: Vec<u8>,
	pub(crate) cap: usize,
	pub(crate) policy: EvictionPolicyKind,
	/// Items force-pushed past a full bucket by negative fixtures (bucket index, item).
	/// Never touched by the cache paths; empty in normal operation.
	pub(crate) overflow: Vec<(usize, Item<K, V>)>,
}

impl<K, V> FlatBuckets<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	fn new(num_buckets: usize, cap: usize, policy: EvictionPolicyKind) -> Self {
		FlatBuckets {
			items: Vec::with_capacity(num_buckets * cap),
			lens: vec![0u8; num_buckets],
			cap,
			policy,
			overflow: Vec::new(),
		}
	}

	#[inline]
	fn num_buckets(&self) -> usize {
		self.lens.len()
	}

	#[inline]
	fn ensure_filled(&mut self, proto: &Item<K, V>) {
		if self.items.is_empty() {
			let n = self.num_buckets() * self.cap;
			self.items.resize(n, proto.clone());
		}
	}

	#[inline]
	pub(crate) fn len(&self, i: usize) -> usize {
		self.lens[i] as usize
	}

	#[inline]
	pub(crate) fn is_full(&self, i: usize) -> bool {
		self.len(i) == self.cap
	}

	/// Live items of bucket `i`.
	#[inline]
	pub(crate) fn slice(&self, i: usize) -> &[Item<K, V>] {
		let l = self.lens[i] as usize;
		if l == 0 {
			return &[];
		}
		let b = i * self.cap;
		&self.items[b..b + l]
	}

	#[inline]
	pub(crate) fn position(&self, i: usize, key: &K) -> Option<usize> {
		find_in(self.slice(i), key)
	}

	/// Index of bucket `i`'s hottest item, or `None` if empty.
	#[inline]
	pub(crate) fn hottest(&self, i: usize) -> Option<usize> {
		let s = self.slice(i);
		if s.is_empty() {
			None
		} else {
			Some(hottest(self.policy, s))
		}
	}

	/// Applies the on-hit policy update to the item at `pos` of bucket `i` and returns it.
	#[inline]
	pub(crate) fn touch(&mut self, i: usize, pos: usize) -> &Item<K, V> {
		let b = i * self.cap;
		let l = self.lens[i] as usize;
		let idx = touch(self.policy, &mut self.items[b..b + l], pos);
		&self.items[b + idx]
	}

	#[inline]
	pub(crate) fn remove_at(&mut self, i: usize, pos: usize) -> Item<K, V> {
		let b = i * self.cap;
		let cap = self.cap;
		remove_at(&mut self.items[b..b + cap], &mut self.lens[i], pos)
	}

	/// `Bucket::put_at` semantics: if the key is already in bucket `i`, update its value
	/// in place and apply the on-hit policy update (ignoring `pos`); otherwise evict the
	/// policy victim if full and insert at `pos`. Returns the evicted item, if any.
	#[inline]
	pub(crate) fn put_at(&mut self, i: usize, item: Item<K, V>, pos: InsertPosition) -> Option<Item<K, V>> {
		if let Some(p) = self.position(i, &item.key) {
			let b = i * self.cap;
			self.items[b + p].value = item.value;
			self.touch(i, p);
			return None;
		}
		self.ensure_filled(&item);
		let b = i * self.cap;
		let cap = self.cap;
		let policy = self.policy;
		let (_, evicted) = insert_at(policy, &mut self.items[b..b + cap], &mut self.lens[i], item, pos);
		evicted
	}

	/// Appends `item` to bucket `i` bypassing eviction; past capacity it goes to the
	/// fixture-only `overflow` list.
	fn force_push(&mut self, i: usize, item: Item<K, V>) {
		if self.len(i) < self.cap {
			self.ensure_filled(&item);
			let at = i * self.cap + self.len(i);
			self.items[at] = item;
			self.lens[i] += 1;
		} else {
			self.overflow.push((i, item));
		}
	}

	fn total(&self) -> usize {
		self.lens.iter().map(|&l| l as usize).sum::<usize>() + self.overflow.len()
	}

	/// All items attributed to bucket `i`, including fixture overflow.
	fn all_items(&self, i: usize) -> impl Iterator<Item = &Item<K, V>> {
		self.slice(i)
			.iter()
			.chain(self.overflow.iter().filter(move |(j, _)| *j == i).map(|(_, it)| it))
	}
}

impl<K, V> Debug for FlatBuckets<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_list()
			.entries((0..self.num_buckets()).map(|i| self.slice(i)))
			.finish()?;
		if !self.overflow.is_empty() {
			write!(f, " overflow={:?}", self.overflow)?;
		}
		Ok(())
	}
}

pub struct SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	/// Normal (target/probing) buckets: one flat item array + per-bucket lengths.
	pub(crate) buckets: FlatBuckets<K, V>,
	pub(crate) segment_size: usize,
	/// Stash buckets: one flat item array + per-bucket lengths.
	pub(crate) stash_buckets: FlatBuckets<K, V>,
	pub(crate) stash_size: usize,
	/// `borrowed[j]` = number of items currently in stash bucket `j` whose primary
	/// stash bucket `s0` is not `j` (overflow borrowed from the left neighbour).
	pub(crate) borrowed: Vec<u8>,
	/// `foreign[i]` = number of items currently in normal bucket `i` whose target index
	/// is not `i` (items placed there as their probing slot).
	pub(crate) foreign: Vec<u8>,
}

impl<K, V> Debug for SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("SegmentCore")
			.field("buckets", &self.buckets)
			.field("segment_size", &self.segment_size)
			.field("stash_buckets", &self.stash_buckets)
			.field("stash_size", &self.stash_size)
			.field("borrowed", &self.borrowed)
			.field("foreign", &self.foreign)
			.finish()
	}
}

impl<K, V> SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	/// Creates a new SegmentCore. Returns `None` if the settings are invalid.
	pub fn try_new(settings: &DashSettings) -> Option<Self> {
		if let Err(problems) = settings.validate() {
			log::error!("SegmentCore.try_new: validation failed: {}", problems.join(", "));
			return None;
		}
		if settings.bucket_size >= u8::MAX as usize {
			log::error!(
				"SegmentCore.try_new: bucket_size={} exceeds the u8 bucket length/counters",
				settings.bucket_size
			);
			return None;
		}
		info!("SegmentCore.try_new: settings={settings:?}");
		Some(SegmentCore {
			segment_size: settings.num_of_normal_buckets,
			stash_size: settings.num_of_stash_buckets,
			buckets: FlatBuckets::new(settings.num_of_normal_buckets, settings.bucket_size, settings.eviction_policy),
			stash_buckets: FlatBuckets::new(settings.num_of_stash_buckets, settings.bucket_size, settings.eviction_policy),
			borrowed: vec![0u8; settings.num_of_stash_buckets],
			foreign: vec![0u8; settings.num_of_normal_buckets],
		})
	}

	/// Creates a new SegmentCore.
	/// # Panics
	/// Panics if initialization fails (e.g., invalid settings).
	pub fn new(settings: &DashSettings) -> Self {
		Self::try_new(settings).expect("SegmentCore::new: invalid settings")
	}

	/// Neighbour stash bucket of the primary stash bucket `s0`: `(s0 + 1) % stash_size`.
	#[inline]
	pub fn stash_neighbor(&self, s0: usize) -> usize {
		(s0 + 1) % self.stash_size
	}

	/// The two candidate stash buckets for `key`: `(s0, s1)`.
	#[inline]
	pub fn stash_indices(&self, key: &K) -> (usize, usize) {
		let s0 = get_index(key, self.stash_size);
		(s0, self.stash_neighbor(s0))
	}

	// ── Read accessors ───────────────────────────────────────────────────────

	/// Live items of normal bucket `i`.
	pub fn normal_items(&self, i: usize) -> &[Item<K, V>] {
		self.buckets.slice(i)
	}

	/// Live items of stash bucket `j`.
	pub fn stash_items(&self, j: usize) -> &[Item<K, V>] {
		self.stash_buckets.slice(j)
	}

	pub fn normal_position(&self, i: usize, key: &K) -> Option<usize> {
		self.buckets.position(i, key)
	}

	pub fn stash_position(&self, j: usize, key: &K) -> Option<usize> {
		self.stash_buckets.position(j, key)
	}

	pub fn normal_is_full(&self, i: usize) -> bool {
		self.buckets.is_full(i)
	}

	pub fn stash_is_full(&self, j: usize) -> bool {
		self.stash_buckets.is_full(j)
	}

	/// Returns `true` if `key` is present in any slot (stash s0/s1, target, or probing).
	pub fn contains_key(&self, key: &K) -> bool {
		let (s0, s1) = self.stash_indices(key);
		let target_idx = get_index(key, self.segment_size);
		let probing_idx = (target_idx + 1) % self.segment_size;
		self.stash_buckets.position(s0, key).is_some()
			|| (s1 != s0 && self.stash_buckets.position(s1, key).is_some())
			|| self.buckets.position(target_idx, key).is_some()
			|| self.buckets.position(probing_idx, key).is_some()
	}

	/// Total items across all normal and stash buckets.
	pub fn total_items(&self) -> usize {
		self.buckets.total() + self.stash_buckets.total()
	}

	/// Maximum items the segment can hold.
	pub fn max_capacity(&self) -> usize {
		(self.segment_size + self.stash_size) * self.buckets.cap
	}

	/// Returns `true` if every item resides in a bucket consistent with its key's derived slot.
	pub fn check_slot_legality(&self) -> bool {
		for i in 0..self.segment_size {
			for item in self.buckets.all_items(i) {
				let target = get_index(&item.key, self.segment_size);
				let probing = (target + 1) % self.segment_size;
				if i != target && i != probing {
					return false;
				}
			}
		}
		for j in 0..self.stash_size {
			for item in self.stash_buckets.all_items(j) {
				let (s0, s1) = self.stash_indices(&item.key);
				if j != s0 && j != s1 {
					return false;
				}
			}
		}
		true
	}

	/// Returns `true` if every `borrowed[j]` matches the recomputed count.
	pub fn check_borrowed_counts(&self) -> bool {
		(0..self.stash_size).all(|j| {
			let actual = self
				.stash_buckets
				.all_items(j)
				.filter(|it| get_index(&it.key, self.stash_size) != j)
				.count();
			actual == self.borrowed[j] as usize
		})
	}

	/// Returns `true` if every `foreign[i]` matches the recomputed count.
	pub fn check_foreign_counts(&self) -> bool {
		(0..self.segment_size).all(|i| {
			let actual = self
				.buckets
				.all_items(i)
				.filter(|it| get_index(&it.key, self.segment_size) != i)
				.count();
			actual == self.foreign[i] as usize
		})
	}

	/// Returns `true` if no individual bucket holds more items than its capacity.
	pub fn check_capacity_bound(&self) -> bool {
		self.buckets.overflow.is_empty()
			&& self.stash_buckets.overflow.is_empty()
			&& self.buckets.lens.iter().all(|&l| l as usize <= self.buckets.cap)
			&& self.stash_buckets.lens.iter().all(|&l| l as usize <= self.stash_buckets.cap)
	}

	/// Push `item` directly into normal bucket `bucket_idx`, bypassing slot derivation,
	/// counters and capacity limits. Intended only for invariant-suite negative fixtures.
	pub fn force_push_normal_bucket(&mut self, bucket_idx: usize, item: Item<K, V>) {
		self.buckets.force_push(bucket_idx, item);
	}

	/// Push `item` directly into stash bucket `stash_idx`, bypassing slot derivation,
	/// counters and capacity limits. Intended only for negative fixtures.
	pub fn force_push_stash_bucket(&mut self, stash_idx: usize, item: Item<K, V>) {
		self.stash_buckets.force_push(stash_idx, item);
	}

	/// Records that an item whose target index is `true_target` just left normal bucket
	/// `source_idx`: decrements `foreign[source_idx]` if it was foreign there.
	#[inline]
	pub fn note_normal_removal(&mut self, source_idx: usize, true_target: usize) {
		if true_target != source_idx {
			debug_assert!(self.foreign[source_idx] > 0, "foreign counter underflow at bucket={source_idx}");
			self.foreign[source_idx] = self.foreign[source_idx].wrapping_sub(1);
		}
	}

	/// Inserts `item` at `pos` into normal bucket `bucket_idx`, keeping `foreign` exact
	/// for both the inserted item and any evictee. Counter-aware fixture helper.
	pub fn put_normal_at_index(
		&mut self,
		bucket_idx: usize,
		pos: InsertPosition,
		item: Item<K, V>,
	) -> Option<Item<K, V>> {
		let target = get_index(&item.key, self.segment_size);
		debug_assert!(bucket_idx == target || bucket_idx == (target + 1) % self.segment_size);
		if target != bucket_idx {
			self.foreign[bucket_idx] = self.foreign[bucket_idx].wrapping_add(1);
		}
		let evicted = self.buckets.put_at(bucket_idx, item, pos);
		if let Some(ref e) = evicted {
			let t = get_index(&e.key, self.segment_size);
			self.note_normal_removal(bucket_idx, t);
		}
		evicted
	}

	/// Decrements `borrowed[stash_idx]` if `removed` was a borrowed item there.
	#[inline]
	fn note_stash_removal(&mut self, stash_idx: usize, removed: &Item<K, V>) {
		if get_index(&removed.key, self.stash_size) != stash_idx {
			debug_assert!(self.borrowed[stash_idx] > 0, "borrowed counter underflow at stash={stash_idx}");
			self.borrowed[stash_idx] = self.borrowed[stash_idx].wrapping_sub(1);
		}
	}

	/// Inserts `item` at `pos` into stash bucket `stash_idx` (one of the key's two stash
	/// candidates). Returns the evicted item, if any. Keeps `borrowed[stash_idx]` exact.
	pub fn put_stash_at_index(
		&mut self,
		stash_idx: usize,
		pos: InsertPosition,
		item: Item<K, V>,
	) -> Option<Item<K, V>> {
		let s0 = get_index(&item.key, self.stash_size);
		debug_assert!(stash_idx == s0 || stash_idx == self.stash_neighbor(s0));
		debug_assert!(self.stash_buckets.position(stash_idx, &item.key).is_none());
		if stash_idx != s0 {
			self.borrowed[stash_idx] = self.borrowed[stash_idx].wrapping_add(1);
		}
		let evicted = self.stash_buckets.put_at(stash_idx, item, pos);
		if let Some(ref e) = evicted {
			debug!("SegmentCore.put_stash_at_index: stash={stash_idx} pos={pos:?} evicted_key={:?}", e.key);
			self.note_stash_removal(stash_idx, e);
		}
		evicted
	}

	/// Removes and returns the item at `item_idx` of stash bucket `stash_idx`, keeping
	/// `borrowed[stash_idx]` exact.
	#[inline]
	pub fn remove_stash_at(&mut self, stash_idx: usize, item_idx: usize) -> Item<K, V> {
		let removed = self.stash_buckets.remove_at(stash_idx, item_idx);
		self.note_stash_removal(stash_idx, &removed);
		removed
	}

	/// Inserts `item` at `pos` in the bucket identified by `slot` (derived from `item.key`).
	/// `Slot::Stash` targets the primary stash bucket `s0`. `Slot::Probing` increments
	/// `foreign[probing_idx]`. Normal-bucket evictees are **not** classified here: the
	/// caller reports each via [`note_normal_removal`](Self::note_normal_removal).
	pub fn put_at(&mut self, slot: Slot, pos: InsertPosition, item: Item<K, V>) -> Option<Item<K, V>> {
		let evicted = match slot {
			Slot::Stash => {
				let idx = get_index(&item.key, self.stash_size);
				let ev = self.stash_buckets.put_at(idx, item, pos);
				if let Some(ref e) = ev {
					self.note_stash_removal(idx, e);
				}
				ev
			}
			Slot::Target => {
				let idx = get_index(&item.key, self.segment_size);
				self.buckets.put_at(idx, item, pos)
			}
			Slot::Probing => {
				let target_idx = get_index(&item.key, self.segment_size);
				let probing_idx = (target_idx + 1) % self.segment_size;
				self.foreign[probing_idx] = self.foreign[probing_idx].wrapping_add(1);
				self.buckets.put_at(probing_idx, item, pos)
			}
		};
		if let Some(ref e) = evicted {
			debug!("SegmentCore.put_at: slot={slot:?} pos={pos:?} evicted_key={:?}", e.key);
		}
		evicted
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;
	use crate::shared::traits::bucket::InsertPosition;
	use crate::test_utils::{find_keys_for_bucket, small_dash_settings};

	#[test]
	fn try_new_rejects_single_normal_bucket() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_normal_buckets = 1;
		assert!(s.validate().is_err());
		assert!(SegmentCore::<i64, i64>::try_new(&s).is_none());
	}

	#[test]
	fn try_new_rejects_zero_stash_buckets() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 0;
		assert!(s.validate().is_err());
		assert!(SegmentCore::<i64, i64>::try_new(&s).is_none());
	}

	// ── flat helpers reproduce Bucket/EvictionPolicy semantics ──────────────

	fn keys(s: &[Item<i64, i64>]) -> Vec<i64> {
		s.iter().map(|i| i.key).collect()
	}

	#[test]
	fn flat_put_front_back_and_lru_eviction() {
		let mut fb = FlatBuckets::<i64, i64>::new(2, 3, EvictionPolicyKind::ClassicLRU);
		fb.put_at(1, Item::new(1, 10), InsertPosition::Back);
		fb.put_at(1, Item::new(2, 20), InsertPosition::Back);
		fb.put_at(1, Item::new(3, 30), InsertPosition::Front);
		assert_eq!(keys(fb.slice(1)), vec![3, 1, 2]);
		assert!(fb.slice(0).is_empty());
		// Touch key 1 (pos 1) → moves to back.
		let p = fb.position(1, &1).unwrap();
		assert_eq!(fb.touch(1, p).key, 1);
		assert_eq!(keys(fb.slice(1)), vec![3, 2, 1]);
		assert_eq!(fb.hottest(1), Some(2));
		// Full: Front insert evicts index 0 first.
		let ev = fb.put_at(1, Item::new(4, 40), InsertPosition::Front);
		assert_eq!(ev.unwrap().key, 3);
		assert_eq!(keys(fb.slice(1)), vec![4, 2, 1]);
		// Existing key: in-place update + touch, no eviction.
		assert!(fb.put_at(1, Item::new(2, 99), InsertPosition::Front).is_none());
		assert_eq!(keys(fb.slice(1)), vec![4, 1, 2]);
		assert_eq!(fb.slice(1)[2].value, 99);
		let r = fb.remove_at(1, 0);
		assert_eq!(r.key, 4);
		assert_eq!(keys(fb.slice(1)), vec![1, 2]);
	}

	#[test]
	fn flat_swap_and_lifo_semantics() {
		let mut fb = FlatBuckets::<i64, i64>::new(1, 3, EvictionPolicyKind::Swap);
		for k in 1..=3 {
			fb.put_at(0, Item::new(k, k), InsertPosition::Back);
		}
		fb.touch(0, 2);
		assert_eq!(keys(fb.slice(0)), vec![1, 3, 2]);
		let mut lifo = FlatBuckets::<i64, i64>::new(1, 2, EvictionPolicyKind::Lifo);
		lifo.put_at(0, Item::new(1, 1), InsertPosition::Back);
		lifo.put_at(0, Item::new(2, 2), InsertPosition::Back);
		let ev = lifo.put_at(0, Item::new(3, 3), InsertPosition::Back);
		assert_eq!(ev.unwrap().key, 2);
		assert_eq!(keys(lifo.slice(0)), vec![1, 3]);
	}

	// ── borrowed counters ────────────────────────────────────────────────────

	#[test]
	fn borrowed_starts_zero() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let core = SegmentCore::<i64, i64>::new(&s);
		assert_eq!(core.borrowed.len(), s.num_of_stash_buckets);
		assert!(core.borrowed.iter().all(|&b| b == 0));
		assert!(core.check_borrowed_counts());
	}

	#[test]
	fn borrowed_tracks_put_and_remove() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 4;
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let key = find_keys_for_bucket(1, 4, 1)[0];
		core.put_stash_at_index(2, InsertPosition::Back, Item::new(key, key));
		assert_eq!(core.borrowed[2], 1);
		assert!(core.check_borrowed_counts());
		let native = find_keys_for_bucket(2, 4, 1)[0];
		core.put_stash_at_index(2, InsertPosition::Back, Item::new(native, native));
		assert_eq!(core.borrowed[2], 1);
		let idx = core.stash_position(2, &key).unwrap();
		let removed = core.remove_stash_at(2, idx);
		assert_eq!(removed.key, key);
		assert_eq!(core.borrowed[2], 0);
		assert!(core.check_borrowed_counts());
	}

	#[test]
	fn borrowed_decrements_on_put_at_stash_eviction() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 4;
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let borrowed_keys = find_keys_for_bucket(1, 4, s.bucket_size);
		for &k in &borrowed_keys {
			core.put_stash_at_index(2, InsertPosition::Back, Item::new(k, k));
		}
		assert_eq!(core.borrowed[2] as usize, s.bucket_size);
		let native = find_keys_for_bucket(2, 4, 1)[0];
		let evicted = core.put_at(Slot::Stash, InsertPosition::Back, Item::new(native, native));
		assert!(evicted.is_some());
		assert_eq!(core.borrowed[2] as usize, s.bucket_size - 1);
		assert!(core.check_borrowed_counts());
	}

	// ── foreign counters ─────────────────────────────────────────────────────

	#[test]
	fn foreign_tracks_probing_put_and_target_put() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let n = s.num_of_normal_buckets;
		let k = find_keys_for_bucket(0, n, 1)[0];
		core.put_at(Slot::Probing, InsertPosition::Back, Item::new(k, k));
		assert_eq!(core.foreign[1], 1);
		assert!(core.check_foreign_counts());
		let native = find_keys_for_bucket(1, n, 1)[0];
		core.put_at(Slot::Target, InsertPosition::Back, Item::new(native, native));
		assert_eq!(core.foreign[1], 1);
		assert!(core.check_foreign_counts());
	}

	#[test]
	fn foreign_put_normal_at_index_tracks_evictee() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let n = s.num_of_normal_buckets;
		let foreign_keys = find_keys_for_bucket(0, n, s.bucket_size);
		for &k in &foreign_keys {
			core.put_normal_at_index(1, InsertPosition::Back, Item::new(k, k));
		}
		assert_eq!(core.foreign[1] as usize, s.bucket_size);
		let native = find_keys_for_bucket(1, n, 1)[0];
		let evicted = core.put_normal_at_index(1, InsertPosition::Back, Item::new(native, native));
		assert!(evicted.is_some());
		assert_eq!(core.foreign[1] as usize, s.bucket_size - 1);
		assert!(core.check_foreign_counts());
	}

	#[test]
	fn contains_key_sees_neighbor_stash_bucket() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 4;
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let key = find_keys_for_bucket(1, 4, 1)[0];
		core.put_stash_at_index(2, InsertPosition::Back, Item::new(key, key));
		assert!(core.contains_key(&key));
		assert!(core.check_slot_legality());
		assert!(core.check_borrowed_counts());
	}

	// ── totals / capacity / legality ────────────────────────────────────────

	#[test]
	fn total_items_counts_normal_and_stash() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		assert_eq!(core.total_items(), 0);
		let normal_key = find_keys_for_bucket(0, s.num_of_normal_buckets, 1)[0];
		core.put_at(Slot::Target, InsertPosition::Back, Item::new(normal_key, normal_key));
		assert_eq!(core.total_items(), 1);
		let stash_key = find_keys_for_bucket(0, s.num_of_stash_buckets, 1)[0];
		core.put_at(Slot::Stash, InsertPosition::Back, Item::new(stash_key, stash_key));
		assert_eq!(core.total_items(), 2);
		assert!(core.stash_position(0, &stash_key).is_some());
	}

	#[test]
	fn max_capacity_matches_settings() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let core = SegmentCore::<i64, i64>::new(&s);
		let expected = (s.num_of_normal_buckets + s.num_of_stash_buckets) * s.bucket_size;
		assert_eq!(core.max_capacity(), expected);
	}

	#[test]
	fn check_slot_legality_detects_wrong_bucket() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		assert!(core.check_slot_legality());
		let bad_key = find_keys_for_bucket(2, s.num_of_normal_buckets, 1)[0];
		core.force_push_normal_bucket(0, Item::new(bad_key, bad_key));
		assert!(!core.check_slot_legality());
	}

	#[test]
	fn check_slot_legality_detects_wrong_stash_bucket() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 4;
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let key = (0i64..200)
			.find(|k| get_index(k, s.num_of_stash_buckets) == 1)
			.expect("no key hashes to stash 1");
		core.force_push_stash_bucket(3, Item::new(key, key));
		assert!(!core.check_slot_legality());
	}

	#[test]
	fn check_capacity_bound_detects_normal_and_stash_overflow() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let keys = find_keys_for_bucket(0, s.num_of_normal_buckets, s.bucket_size + 1);
		for &k in &keys[..s.bucket_size] {
			core.force_push_normal_bucket(0, Item::new(k, k));
		}
		assert!(core.check_capacity_bound());
		core.force_push_normal_bucket(0, Item::new(keys[s.bucket_size], 0));
		assert!(!core.check_capacity_bound());
		assert_eq!(core.total_items(), s.bucket_size + 1);

		let mut core = SegmentCore::<i64, i64>::new(&s);
		let keys = find_keys_for_bucket(0, s.num_of_stash_buckets, s.bucket_size + 1);
		for k in keys {
			core.force_push_stash_bucket(0, Item::new(k, k));
		}
		assert!(!core.check_capacity_bound());
	}

	#[test]
	fn force_push_normal_bucket_item_appears_in_requested_bucket() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let key = find_keys_for_bucket(0, s.num_of_normal_buckets, 1)[0];
		core.force_push_normal_bucket(3, Item::new(key, key));
		assert!(core.normal_position(3, &key).is_some());
		assert_eq!(core.total_items(), 1);
	}
}
