use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 64;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;
const DEMOTE_SAMPLE: usize = 5;
/// Minimum *pre-increment* sketch estimate a probationary node must carry to be
/// admitted into spare protected capacity for free. A node observed fewer than
/// this many times before the current access has no demonstrated prior reuse and
/// is kept probationary (its recency refreshed) instead of consuming a protected
/// slot. Evaluated pre-increment so the current access does not satisfy the floor
/// on its own.
const PROMOTE_FLOOR: u32 = 2;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	protected: bool,
}

/// Two-segment SLRU over a single arena (`nodes`).
///
/// The resident set is split into a *probationary* region and a *protected*
/// region. Newly admitted keys land in the probationary tail. A hit promotes
/// the node into the protected tail; if the protected segment now exceeds its
/// cap, a frequency-ranked victim drawn from a small protected-LRU window is
/// demoted back to the probationary tail. Eviction only ever considers the
/// probationary head as victim, so proven-hot keys living in the protected
/// tier cannot be trampled directly by admission churn.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	// probationary segment (head = LRU / victim end, tail = MRU end)
	prob_head: Option<usize>,
	prob_tail: Option<usize>,
	// protected segment (head = LRU / demotion end, tail = MRU end)
	prot_head: Option<usize>,
	prot_tail: Option<usize>,
	protected_len: usize,
	protected_cap: usize,
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
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		// Protected tier holds floor(capacity * 0.8); the remainder (at least 1)
		// is probationary so there is always a victim when the cache is full.
		let protected_cap = (capacity * 4) / 5;
		let protected_cap = protected_cap.min(capacity.saturating_sub(1));
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} protected_cap={protected_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			prob_head: None,
			prob_tail: None,
			prot_head: None,
			prot_tail: None,
			protected_len: 0,
			protected_cap,
			capacity,
			sketch: CountMinSketch::new(width),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Capture the estimate *before* incrementing so the promotion floor sees
		// the key's prior frequency, excluding the current access.
		let prior = self.sketch.estimate(key);
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		self.promote(idx, prior);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		// Capture the estimate *before* incrementing so a re-hit promotion sees
		// the key's prior frequency, excluding the current access.
		let prior = self.sketch.estimate(&key);
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.promote(idx, prior);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				protected: false,
			});
			self.push_back_prob(idx);
			self.map.insert(key, idx);
			return;
		}

		// Cache full: the only eviction candidate is the probationary head.
		let victim_idx = self
			.prob_head
			.expect("TinyLfuCache: full cache has no probationary victim");
		let victim_key = self.nodes[victim_idx].key.clone();
		let victim_freq = self.sketch.estimate(&victim_key);
		let incoming_freq = self.sketch.estimate(&key);
		if incoming_freq > victim_freq {
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.push_back_prob(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	/// Promote `idx` on a hit.
	///
	/// - A node already in the protected segment stays protected: it is simply
	///   moved to the protected tail (MRU), no contest.
	/// - A probationary node is promoted into the protected tail while the
	///   protected segment has spare room *and* the key has demonstrated prior
	///   reuse (`prior_freq >= PROMOTE_FLOOR`, evaluated pre-increment). A node
	///   below the floor is kept probationary (recency refreshed) so it cannot
	///   claim a protected slot on a first-touch.
	/// - When promoting a probationary node would overflow `protected_cap`, the
	///   promotion becomes a frequency contest against the protected LRU head:
	///   only if the promotee's sketch estimate is strictly greater do we admit
	///   the promotee. When admitted, the demotion victim is not the LRU head
	///   unconditionally but the coldest node in a small protected-LRU window
	///   (`sample_prot_demotee`). Otherwise the promotee is re-inserted at the
	///   probationary tail (recency refreshed) and the protected segment is left
	///   untouched.
	fn promote(&mut self, idx: usize, prior_freq: u32) {
		if self.nodes[idx].protected {
			// Re-hit of a protected node: it always stays protected; refresh its
			// recency by moving it to the protected tail.
			self.unlink(idx);
			self.push_back_prot(idx);
			return;
		}

		// Node currently lives in the probationary segment.
		self.unlink(idx);

		if self.protected_len < self.protected_cap {
			// Spare room in the protected tier. Admit only keys with demonstrated
			// prior reuse; otherwise keep the node probationary (recency refreshed)
			// so a first-touch cannot consume a protected slot for free.
			if prior_freq >= PROMOTE_FLOOR {
				self.push_back_prot(idx);
				self.protected_len += 1;
			} else {
				self.push_back_prob(idx);
			}
			return;
		}

		// Protected tier is full. Gate promotion on estimated frequency: only
		// admit the promotee when it is strictly hotter than the protected LRU
		// head. The admission test is unchanged; what changes is which protected
		// node is demoted once the promotee wins.
		match self.prot_head {
			Some(head_idx) => {
				let promotee_freq = self.sketch.estimate(&self.nodes[idx].key);
				let head_freq = self.sketch.estimate(&self.nodes[head_idx].key);
				if promotee_freq > head_freq {
					// Demote the coldest node in a small protected-LRU window
					// rather than the LRU head unconditionally.
					let demotee = self.sample_prot_demotee();
					self.unlink(demotee);
					self.protected_len -= 1;
					self.push_back_prob(demotee);
					self.push_back_prot(idx);
					self.protected_len += 1;
				} else {
					// Not hot enough to displace the protected head: keep the
					// promotee probationary but refresh its recency.
					self.push_back_prob(idx);
				}
			}
			None => {
				// No protected segment exists (e.g. protected_cap == 0): the node
				// stays probationary.
				self.push_back_prob(idx);
			}
		}
	}

	/// Choose the demotion victim from a small window at the protected LRU end.
	///
	/// Walk up to `DEMOTE_SAMPLE` nodes starting at `prot_head` along the
	/// protected `next` chain (LRU -> MRU) and return the index with the lowest
	/// sketch estimate. Ties are broken toward the most-LRU node via a strict
	/// `<` (the most-LRU candidate is seeded first and only displaced when a
	/// later node is strictly colder). The walk stays within the protected list
	/// because probationary and protected nodes are separate chains.
	fn sample_prot_demotee(&self) -> usize {
		let mut best = self
			.prot_head
			.expect("sample_prot_demotee: empty protected segment");
		let mut best_freq = self.sketch.estimate(&self.nodes[best].key);
		let mut cur = self.nodes[best].next;
		let mut count = 1;
		while count < DEMOTE_SAMPLE {
			match cur {
				Some(idx) => {
					let freq = self.sketch.estimate(&self.nodes[idx].key);
					if freq < best_freq {
						best_freq = freq;
						best = idx;
					}
					cur = self.nodes[idx].next;
					count += 1;
				}
				None => break,
			}
		}
		best
	}

	/// Unlink `idx` from whichever segment it currently belongs to (per its
	/// `protected` flag). Does not adjust `protected_len`.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let in_protected = self.nodes[idx].protected;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				if in_protected {
					self.prot_head = next;
				} else {
					self.prob_head = next;
				}
			}
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => {
				if in_protected {
					self.prot_tail = prev;
				} else {
					self.prob_tail = prev;
				}
			}
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	fn push_back_prob(&mut self, idx: usize) {
		self.nodes[idx].protected = false;
		self.nodes[idx].prev = self.prob_tail;
		self.nodes[idx].next = None;
		match self.prob_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prob_head = Some(idx),
		}
		self.prob_tail = Some(idx);
	}

	fn push_back_prot(&mut self, idx: usize) {
		self.nodes[idx].protected = true;
		self.nodes[idx].prev = self.prot_tail;
		self.nodes[idx].next = None;
		match self.prot_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prot_head = Some(idx),
		}
		self.prot_tail = Some(idx);
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	counts: Vec<Vec<u32>>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let depth = SKETCH_DEPTH;
		let counts = vec![vec![0; width]; depth];
		Self { width, depth, counts }
	}

	fn increment<K: Hash>(&mut self, key: &K) {
		for row in 0..self.depth {
			let pos = self.position(key, row as u64);
			let counter = &mut self.counts[row][pos];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
	}

	fn estimate<K: Hash>(&self, key: &K) -> u32 {
		self
			.counts
			.iter()
			.enumerate()
			.map(|(row, _)| self.counts[row][self.position(key, row as u64)])
			.min()
			.unwrap_or(0)
	}

	fn position<K: Hash>(&self, key: &K, seed: u64) -> usize {
		let mut hasher = DefaultHasher::new();
		hasher.write_u64(seed);
		key.hash(&mut hasher);
		(hasher.finish() as usize) & (self.width - 1)
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
	fn eviction_prefers_hot_items() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // promote key 1 into the protected tier
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

	#[test]
	fn protected_key_shielded_from_probationary_eviction() {
		// capacity 4 -> protected_cap 3, probationary 1.
		let mut cache = TinyLfuCache::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		// Promote 1,2,3 into the protected tier; 4 stays probationary.
		cache.get(&1);
		cache.get(&2);
		cache.get(&3);
		// Hammer a hot newcomer so it out-scores the probationary victim (4).
		for _ in 0..5 {
			cache.get(&9);
		}
		cache.put(9, 90);

		// 4 (probationary head) is the only victim; protected keys survive.
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&9).unwrap(), 90);
	}
}
