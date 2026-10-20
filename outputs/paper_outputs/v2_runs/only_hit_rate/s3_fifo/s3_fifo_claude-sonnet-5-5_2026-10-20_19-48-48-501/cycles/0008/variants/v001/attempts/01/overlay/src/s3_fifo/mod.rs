use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// S3-FIFO variant: adaptive small queue (starts ~5%, tuned from ghost-hit
// feedback per window of `capacity` requests), ghost of ~0.5x main size,
// main-queue victims are also recorded in the ghost, ghost hits insert into
// main with freq=0. Small-to-main promotion requires freq >= PROMOTE_FREQ (2).
const MAX_FREQ: u8 = 3;
const PROMOTE_FREQ: u8 = 2;

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
	freq: u8,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self { key, value, freq: 0 }
	}
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost: VecDeque<K>,
	small_size: usize,
	main_size: usize,
	ghost_size: usize,
	capacity: usize,
	window_requests: usize,
	ghost_hits_window: usize,
	small_evict_to_main_window: usize,
}

impl<K: PartialEq + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 20).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		// Ghost length stays fixed at its initial value.
		let ghost_size = (main_size / 2).max(1);
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(ghost_size),
			small_size,
			main_size,
			ghost_size,
			capacity,
			window_requests: 0,
			ghost_hits_window: 0,
			small_evict_to_main_window: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)
	}

	fn tick(&mut self) {
		self.window_requests += 1;
		if self.window_requests >= self.capacity.max(1) {
			self.end_window();
		}
	}

	fn end_window(&mut self) {
		let reqs = self.window_requests.max(1);
		let cap = self.capacity;
		if cap >= 2 {
			let step = (cap / 100).max(1);
			let min_small = (cap / 50).max(1);
			let max_small = (cap / 5).max(1).min(cap - 1);
			let mut target = self.small_size;
			if self.ghost_hits_window * 50 > reqs {
				target = (target + step).min(max_small);
			} else if self.ghost_hits_window == 0 && self.small_evict_to_main_window * 200 < reqs {
				target = target.saturating_sub(step).max(min_small);
			}
			if target != self.small_size {
				self.small_size = target;
				self.main_size = cap - target;
				// Restore size bounds for the new split.
				while self.main.len() > self.main_size {
					self.evict_main();
				}
				while self.small.len() > self.small_size {
					self.evict_small();
				}
			}
		}
		self.window_requests = 0;
		self.ghost_hits_window = 0;
		self.small_evict_to_main_window = 0;
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.tick();
		let entry = self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)?;
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(entry) = self.find_entry_mut(&key) {
			entry.value = value;
			return;
		}

		let in_ghost = self.ghost.iter().any(|ghost_key| ghost_key == &key);
		if in_ghost {
			self.ghost_hits_window += 1;
			self.ghost.retain(|ghost_key| ghost_key != &key);
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.main.push_front(Entry::new(key, value));
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.small.push_front(Entry::new(key, value));
	}

	fn push_ghost(&mut self, key: K) {
		while self.ghost.len() >= self.ghost_size {
			self.ghost.pop_back();
		}
		self.ghost.push_front(key);
	}

	fn evict_main(&mut self) {
		while let Some(mut tail) = self.main.pop_back() {
			if tail.freq > 0 {
				tail.freq -= 1;
				self.main.push_front(tail);
			} else {
				self.push_ghost(tail.key);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(tail) = self.small.pop_back() {
			if tail.freq >= PROMOTE_FREQ {
				self.small_evict_to_main_window += 1;
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(tail);
			} else {
				self.push_ghost(tail.key);
			}
		}
	}
}

impl<K: PartialEq + Eq + std::hash::Hash + Clone + Debug, V: Debug> framework::Cache<K, V> for S3Fifo<K, V> {
	fn put(&mut self, key: K, value: V) {
		S3Fifo::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		S3Fifo::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn put_then_get_finds_value() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.small.len() + cache.main.len() <= 4);
	}
}
