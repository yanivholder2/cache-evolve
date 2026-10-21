use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
// The unrolled sketch paths below hard-code four rows.
const _: () = assert!(SKETCH_DEPTH == 4);
const SKETCH_WIDTH_FACTOR: usize = 8;
const MIN_SKETCH_WIDTH: usize = 16;
/// Saturation point of the `u16` counters, expressed in the `u32` comparison domain.
const MAX_COUNT: u32 = u16::MAX as u32;

/// Sentinel for "no link" in the u32-indexed intrusive list, and for an empty index slot.
const NIL: u32 = u32::MAX;

const FX_SEED: u64 = 0x517c_c1b7_2722_0a95;

/// Minimal FxHash-style hasher (local; no external crates).
#[derive(Default, Clone, Copy)]
pub struct FxLikeHasher {
	state: u64,
}

impl FxLikeHasher {
	#[inline(always)]
	fn add(&mut self, word: u64) {
		self.state = (self.state.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FxLikeHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for c in &mut chunks {
			let mut buf = [0u8; 8];
			buf.copy_from_slice(c);
			self.add(u64::from_le_bytes(buf));
		}
		let rem = chunks.remainder();
		if !rem.is_empty() {
			let mut buf = [0u8; 8];
			buf[..rem.len()].copy_from_slice(rem);
			self.add(u64::from_le_bytes(buf));
		}
	}

	#[inline]
	fn write_u8(&mut self, i: u8) {
		self.add(i as u64);
	}

	#[inline]
	fn write_u16(&mut self, i: u16) {
		self.add(i as u64);
	}

	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.add(i as u64);
	}

	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.add(i);
	}

	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.add(i as u64);
	}

	#[inline]
	fn finish(&self) -> u64 {
		self.state
	}
}

/// murmur3 fmix64 finaliser for avalanche.
#[inline(always)]
fn fmix64(mut h: u64) -> u64 {
	h ^= h >> 33;
	h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
	h ^= h >> 33;
	h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
	h ^= h >> 33;
	h
}

/// Hash a key once with the fast hasher plus a finaliser mix.
#[inline]
fn key_hash<K: Hash>(key: &K) -> u64 {
	let mut hasher = FxLikeHasher::default();
	key.hash(&mut hasher);
	fmix64(hasher.finish())
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	/// Open-addressing index (linear probing, load factor <= 0.5). Each slot holds a node
	/// index into `nodes` or `NIL`; keys live only in `nodes[idx].key`.
	slots: Vec<u32>,
	/// `slots.len() - 1` (`slots.len()` is a power of two).
	mask: usize,
	nodes: Vec<Node<K, V>>,
	head: u32,
	tail: u32,
	capacity: usize,
	sketch: CountMinSketch,
}

impl<K, V> TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("TinyLfuCache.try_new: capacity must be > 0");
			return None;
		}
		if capacity >= u32::MAX as usize {
			log::error!(
				"TinyLfuCache.try_new: capacity={capacity} must be < u32::MAX for u32 node indices"
			);
			return None;
		}
		let Some(table_size) = capacity
			.checked_mul(2)
			.and_then(|n| n.checked_next_power_of_two())
		else {
			log::error!("TinyLfuCache.try_new: capacity={capacity} too large for index table");
			return None;
		};
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} index_slots={table_size}"
		);
		Some(Self {
			slots: vec![NIL; table_size],
			mask: table_size - 1,
			nodes: Vec::with_capacity(capacity),
			head: NIL,
			tail: NIL,
			capacity,
			sketch: CountMinSketch::new(width),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u16>()`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let h = key_hash(key);
		self.sketch.increment_only(h);
		let (_, idx) = self.find(h, key)?;
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx as usize].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let h = key_hash(&key);
		// Conservative update returns the key's post-increment estimate.
		let incoming_freq = self.sketch.increment_hash(h);
		if let Some((_, idx)) = self.find(h, &key) {
			self.nodes[idx as usize].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key,
				value,
				prev: NIL,
				next: NIL,
			});
			self.push_back(idx);
			self.insert_slot(h, idx);
			return;
		}

		let victim_idx = self.head;
		debug_assert!(victim_idx != NIL, "TinyLfuCache: full cache has no victim");
		let vh = key_hash(&self.nodes[victim_idx as usize].key);
		let victim_freq = self.sketch.estimate_hash(vh);
		if incoming_freq > victim_freq {
			let (victim_slot, found) = self
				.find(vh, &self.nodes[victim_idx as usize].key)
				.expect("TinyLfuCache: resident victim missing from index");
			debug_assert_eq!(found, victim_idx);
			self.remove_slot(victim_slot);
			self.unlink(victim_idx);
			let node = &mut self.nodes[victim_idx as usize];
			node.key = key;
			node.value = value;
			self.push_back(victim_idx);
			self.insert_slot(h, victim_idx);
		}
	}

	/// Home slot for hash `h`. Uses the high half so slot bits are independent of the
	/// sketch's `h1` (low half).
	#[inline(always)]
	fn home(&self, h: u64) -> usize {
		((h >> 32) as usize) & self.mask
	}

	/// Linear probe from `h`'s home slot until `NIL`; returns `(slot, node index)` on match.
	#[inline]
	fn find(&self, h: u64, key: &K) -> Option<(usize, u32)> {
		let mut s = self.home(h);
		loop {
			// SAFETY: `s` is masked with `mask = slots.len() - 1`, so `s < slots.len()`.
			let idx = unsafe { *self.slots.get_unchecked(s) };
			if idx == NIL {
				return None;
			}
			if self.nodes[idx as usize].key == *key {
				return Some((s, idx));
			}
			s = (s + 1) & self.mask;
		}
	}

	/// Store `idx` at the first empty slot on `h`'s probe sequence.
	#[inline]
	fn insert_slot(&mut self, h: u64, idx: u32) {
		let mut s = self.home(h);
		// Load factor <= 0.5 guarantees an empty slot exists.
		while self.slots[s] != NIL {
			s = (s + 1) & self.mask;
		}
		self.slots[s] = idx;
	}

	/// Backward-shift deletion (Knuth algorithm R) of the entry at `slot`.
	fn remove_slot(&mut self, slot: usize) {
		let mask = self.mask;
		let mut hole = slot;
		let mut j = slot;
		loop {
			j = (j + 1) & mask;
			let idx = self.slots[j];
			if idx == NIL {
				break;
			}
			let k = ((key_hash(&self.nodes[idx as usize].key) >> 32) as usize) & mask;
			// The entry at `j` must stay put iff its home `k` lies cyclically in `(hole, j]`.
			let stays = if hole <= j {
				hole < k && k <= j
			} else {
				hole < k || k <= j
			};
			if !stays {
				self.slots[hole] = idx;
				hole = j;
			}
		}
		self.slots[hole] = NIL;
	}

	fn unlink(&mut self, idx: u32) {
		let i = idx as usize;
		let prev = self.nodes[i].prev;
		let next = self.nodes[i].next;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.head = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else {
			self.tail = prev;
		}
		self.nodes[i].prev = NIL;
		self.nodes[i].next = NIL;
	}

	fn push_back(&mut self, idx: u32) {
		let i = idx as usize;
		self.nodes[i].prev = self.tail;
		self.nodes[i].next = NIL;
		if self.tail != NIL {
			self.nodes[self.tail as usize].next = idx;
		} else {
			self.head = idx;
		}
		self.tail = idx;
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	/// Flattened `SKETCH_DEPTH * width` saturating `u16` counters; row `r` occupies
	/// `[r * width, (r + 1) * width)`.
	counts: Vec<u16>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		debug_assert!(width.is_power_of_two());
		let counts = vec![0u16; SKETCH_DEPTH * width];
		Self { width, counts }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u16>()`.
	fn heap_bytes(&self) -> usize {
		SKETCH_DEPTH * self.width * std::mem::size_of::<u16>()
	}

	/// Flat index for `row`, derived by double hashing from one 64-bit hash.
	#[inline(always)]
	fn index(&self, h1: u32, h2: u32, row: usize) -> usize {
		let col = (h1.wrapping_add((row as u32).wrapping_mul(h2)) as usize) & (self.width - 1);
		row * self.width + col
	}

	/// The key's four flat counter indices, one per row.
	#[inline(always)]
	fn indices(&self, h: u64) -> [usize; 4] {
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		let i0 = self.index(h1, h2, 0);
		let i1 = self.index(h1, h2, 1);
		let i2 = self.index(h1, h2, 2);
		let i3 = self.index(h1, h2, 3);
		debug_assert!(i0 < self.counts.len());
		debug_assert!(i1 < self.counts.len());
		debug_assert!(i2 < self.counts.len());
		debug_assert!(i3 < self.counts.len());
		[i0, i1, i2, i3]
	}

	/// Fused single-pass conservative update. Loads the four counters once into
	/// registers, computes the minimum, then writes back from the locals, bumping only
	/// counters equal to the minimum (branch-free). Saturates at `MAX_COUNT`
	/// (`u16::MAX`): if the minimum is already at the max, nothing changes.
	/// Returns the key's new estimate (`min + 1`, saturating).
	#[inline(always)]
	fn increment_hash(&mut self, h: u64) -> u32 {
		let [i0, i1, i2, i3] = self.indices(h);
		// SAFETY: `index` masks the column with `width - 1` (width is a power of two), so
		// `col < width`, and `row < SKETCH_DEPTH`, giving `row * width + col <
		// SKETCH_DEPTH * width == counts.len()`. Each index lies in a distinct row's
		// disjoint range `[r * width, (r + 1) * width)`, so the four indices never alias
		// and the loads/stores below are independent.
		unsafe {
			let c0 = *self.counts.get_unchecked(i0) as u32;
			let c1 = *self.counts.get_unchecked(i1) as u32;
			let c2 = *self.counts.get_unchecked(i2) as u32;
			let c3 = *self.counts.get_unchecked(i3) as u32;
			let min = c0.min(c1).min(c2.min(c3));
			// 1 unless the minimum is saturated; guards the `ci == min` bump against overflow.
			let inc = (min < MAX_COUNT) as u32;
			*self.counts.get_unchecked_mut(i0) = (c0 + (((c0 == min) as u32) & inc)) as u16;
			*self.counts.get_unchecked_mut(i1) = (c1 + (((c1 == min) as u32) & inc)) as u16;
			*self.counts.get_unchecked_mut(i2) = (c2 + (((c2 == min) as u32) & inc)) as u16;
			*self.counts.get_unchecked_mut(i3) = (c3 + (((c3 == min) as u32) & inc)) as u16;
			min + inc
		}
	}

	/// Same update as `increment_hash`, discarding the estimate (used on `get`).
	#[inline(always)]
	fn increment_only(&mut self, h: u64) {
		let _ = self.increment_hash(h);
	}

	#[inline(always)]
	fn estimate_hash(&self, h: u64) -> u32 {
		let [i0, i1, i2, i3] = self.indices(h);
		// SAFETY: indices are in bounds; see `increment_hash`.
		unsafe {
			let c0 = *self.counts.get_unchecked(i0) as u32;
			let c1 = *self.counts.get_unchecked(i1) as u32;
			let c2 = *self.counts.get_unchecked(i2) as u32;
			let c3 = *self.counts.get_unchecked(i3) as u32;
			c0.min(c1).min(c2.min(c3))
		}
	}
}

impl<K, V> framework::Cache<K, V> for TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	fn put(&mut self, key: K, value: V) {
		TinyLfuCache::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		TinyLfuCache::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashSet;

	#[test]
	fn get_miss_returns_none() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = TinyLfuCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn node_is_compact() {
		assert_eq!(std::mem::size_of::<Node<u64, u64>>(), 24);
	}

	/// Every resident node is findable at its own index, the table holds exactly the
	/// residents, and every key in `universe` that is not resident is absent.
	fn check_index(cache: &TinyLfuCache<i64, i64>, universe: impl Iterator<Item = i64>) {
		let occupied = cache.slots.iter().filter(|&&s| s != NIL).count();
		assert_eq!(occupied, cache.nodes.len());
		let resident: HashSet<i64> = cache.nodes.iter().map(|n| n.key).collect();
		assert_eq!(resident.len(), cache.nodes.len(), "duplicate resident keys");
		for (i, n) in cache.nodes.iter().enumerate() {
			let (_, idx) = cache
				.find(key_hash(&n.key), &n.key)
				.unwrap_or_else(|| panic!("resident key {} not findable", n.key));
			assert_eq!(idx as usize, i);
		}
		for k in universe {
			if !resident.contains(&k) {
				assert!(cache.find(key_hash(&k), &k).is_none(), "evicted key {k} findable");
			}
		}
	}

	#[test]
	fn index_churn_keeps_residents_findable_and_evicted_absent() {
		let cap = 64;
		let mut cache = TinyLfuCache::<i64, i64>::new(cap);
		let universe = 0i64..1000;
		let mut evictions = 0usize;
		// Skewed, shifting workload so many admissions (and thus index removals) happen.
		for step in 0..20_000i64 {
			let key = (step * 7919 + (step / 500) * 131) % 1000;
			let key = if step % 3 == 0 { key % 50 + (step / 2000) * 50 } else { key };
			let before: HashSet<i64> = cache.nodes.iter().map(|n| n.key).collect();
			match cache.get(&key) {
				Some(&v) => assert_eq!(v, key * 3),
				None => cache.put(key, key * 3),
			}
			let after: HashSet<i64> = cache.nodes.iter().map(|n| n.key).collect();
			evictions += before.difference(&after).count();
			if step % 97 == 0 {
				check_index(&cache, universe.clone());
			}
		}
		check_index(&cache, universe);
		assert!(evictions > cap, "workload did not churn: {evictions} evictions");
	}

	#[test]
	fn backward_shift_deletion_with_forced_collisions() {
		let cap = 16;
		let mut cache = TinyLfuCache::<i64, i64>::new(cap);
		let mask = cache.mask;
		let home = |k: i64| ((key_hash(&k) >> 32) as usize) & mask;
		// Cluster around the wrap-around boundary: keys homed at the last slot and slot 0.
		let mut keys: Vec<i64> = Vec::new();
		let mut k = 0i64;
		while keys.iter().filter(|&&x| home(x) == mask).count() < 7 {
			if home(k) == mask {
				keys.push(k);
			}
			k += 1;
		}
		k = 0;
		while keys.iter().filter(|&&x| home(x) == 0).count() < 4 {
			if home(k) == 0 && !keys.contains(&k) {
				keys.push(k);
			}
			k += 1;
		}
		k = 0;
		while keys.iter().filter(|&&x| home(x) == 1).count() < 3 {
			if home(k) == 1 && !keys.contains(&k) {
				keys.push(k);
			}
			k += 1;
		}
		assert!(keys.len() <= cap);
		for &k in &keys {
			cache.put(k, k + 1);
		}
		check_index(&cache, keys.iter().copied());

		// Delete in an interleaved order, checking every remaining key after each delete.
		let order: Vec<usize> = (0..keys.len()).map(|i| (i * 5) % keys.len()).collect();
		let mut removed: HashSet<i64> = HashSet::new();
		for &oi in &order {
			let k = keys[oi];
			if removed.contains(&k) {
				continue;
			}
			let (slot, _) = cache.find(key_hash(&k), &k).expect("key present before delete");
			cache.remove_slot(slot);
			removed.insert(k);
			for &other in &keys {
				let found = cache.find(key_hash(&other), &other);
				if removed.contains(&other) {
					assert!(found.is_none(), "removed key {other} still findable");
				} else {
					let (_, idx) = found.unwrap_or_else(|| panic!("key {other} lost after delete"));
					assert_eq!(cache.nodes[idx as usize].key, other);
				}
			}
		}
		assert!(cache.slots.iter().all(|&s| s == NIL));
	}

	#[test]
	fn conservative_update_bumps_only_minimum_counters() {
		let mut s = CountMinSketch::new(16);
		let h = key_hash(&7u64);
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		// Inflate one row's counter so it sits above the others.
		let i0 = s.index(h1, h2, 0);
		s.counts[i0] = 5;
		assert_eq!(s.increment_hash(h), 1);
		assert_eq!(s.counts[i0], 5);
		assert_eq!(s.estimate_hash(h), 1);
		assert_eq!(s.increment_hash(h), 2);
		assert_eq!(s.counts[i0], 5);
		s.increment_only(h);
		assert_eq!(s.estimate_hash(h), 3);
		assert_eq!(s.counts[i0], 5);
	}

	#[test]
	fn conservative_update_saturates() {
		let mut s = CountMinSketch::new(16);
		for c in s.counts.iter_mut() {
			*c = u16::MAX;
		}
		let h = key_hash(&3u64);
		assert_eq!(s.increment_hash(h), u16::MAX as u32);
		assert!(s.counts.iter().all(|&c| c == u16::MAX));
	}

	#[test]
	fn eviction_prefers_hot_items() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // make key 1 hotter than key 2
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn low_frequency_new_item_is_rejected() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30);

		assert!(cache.get(&2).is_some());
		assert!(cache.get(&3).is_none());
	}

	#[test]
	fn update_promotes_existing_item_and_admits_hot_arrivals() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}
}
