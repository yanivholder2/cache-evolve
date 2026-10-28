// Count-min frequency sketch with 4-bit counters (TinyLFU-style admission).
// Counters are packed 16 per u64 word; depth 4 (each row picks one word and
// one nibble from an independently-seeded 64-bit mix of the key hash).
// Ageing: after `10 * capacity` increments every counter is halved.

const DEPTH: usize = 4;
const SEEDS: [u64; DEPTH] = [
	0x97cb_3127_dd1c_a3a1,
	0xc3a5_c85c_97cb_3127,
	0xb492_b66f_be98_f273,
	0x9ae1_6a3b_2f90_404f,
];
const RESET_MASK: u64 = 0x7777_7777_7777_7777;

#[inline]
pub fn splitmix64(mut x: u64) -> u64 {
	x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
	x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
	x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
	x ^ (x >> 31)
}

#[derive(Debug)]
pub struct FrequencySketch {
	table: Vec<u64>,
	mask: usize,
	count: usize,
	sample_size: usize,
}

impl FrequencySketch {
	pub fn new(capacity: usize) -> Self {
		let width = capacity.max(1).next_power_of_two();
		Self {
			table: vec![0u64; width],
			mask: width - 1,
			count: 0,
			sample_size: capacity.max(1).saturating_mul(10),
		}
	}

	/// Returns (word index, bit shift) for row `i`.
	#[inline]
	fn slot(&self, h: u64, i: usize) -> (usize, u32) {
		let x = splitmix64(h ^ SEEDS[i]);
		let word = (x as usize) & self.mask;
		let shift = ((x >> 60) as u32) << 2;
		(word, shift)
	}

	/// Increments every row's counter for `h` (each capped at 15).
	pub fn increment(&mut self, h: u64) {
		let mut slots = [(0usize, 0u32); DEPTH];
		for i in 0..DEPTH {
			let s = self.slot(h, i);
			slots[i] = s;
			// Rows that land on the same counter bump it only once.
			if slots[..i].contains(&s) {
				continue;
			}
			let (w, shift) = s;
			let c = (self.table[w] >> shift) & 0xf;
			if c < 15 {
				self.table[w] += 1u64 << shift;
			}
		}
		self.count += 1;
		if self.count >= self.sample_size {
			self.reset();
		}
	}

	/// Estimated frequency: minimum counter across rows.
	pub fn frequency(&self, h: u64) -> u8 {
		let mut min = 15u64;
		for i in 0..DEPTH {
			let (w, shift) = self.slot(h, i);
			let c = (self.table[w] >> shift) & 0xf;
			if c < min {
				min = c;
			}
		}
		min as u8
	}

	/// Halves every counter and the sample count.
	fn reset(&mut self) {
		for w in self.table.iter_mut() {
			*w = (*w >> 1) & RESET_MASK;
		}
		self.count /= 2;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn counts_and_caps() {
		let mut s = FrequencySketch::new(1024);
		assert_eq!(s.frequency(42), 0);
		for _ in 0..5 {
			s.increment(42);
		}
		assert_eq!(s.frequency(42), 5);
		for _ in 0..50 {
			s.increment(42);
		}
		assert!(s.frequency(42) <= 15);
	}

	#[test]
	fn ageing_halves() {
		let mut s = FrequencySketch::new(1);
		// sample_size = 10; 9 increments stay below the threshold
		for _ in 0..9 {
			s.increment(7);
		}
		assert_eq!(s.frequency(7), 9);
		s.increment(7); // 10th triggers reset: 10 -> 5
		assert_eq!(s.frequency(7), 5);
	}
}
