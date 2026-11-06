use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::trace;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Newly admitted items are inserted at `len / INSERT_FRACTION_DIV` (measured
/// from the LRU end, index 0) instead of the MRU tail. LRU-insertion-point
/// scheme: a new item must be hit again before it drifts the remaining
/// ~1/INSERT_FRACTION_DIV of the bucket down to the eviction end.
const INSERT_FRACTION_DIV: usize = 4;

/// Fixed bimodal period used when the per-item tag feedback is unavailable
/// (eviction policies other than ClassicLRU may use `lfu_counter` themselves).
const BIMODAL_PERIOD: u32 = 32;

/// Saturating per-bucket policy selector steering the MRU-sample rate.
const PSEL_MAX: u8 = 15;

// The tag packs a 32-bit stamp at bits 17..=48, so `usize` must be 64-bit.
const _: () = assert!(usize::BITS >= 64);

const SAMPLED_BIT: usize = 1;
const INS_SHIFT: u32 = 1;
const INS_MASK: usize = 0xFFFF;
const STAMP_SHIFT: u32 = 17;
const STAMP_MASK: usize = 0xFFFF_FFFF;

/// Encode a sample tag: bit 0 = sampled, bits 1..=16 = would-be probation
/// depth (capped at 0xFFFF), bits 17..=48 = bucket miss_count at admission.
#[inline]
fn tag(stamp: u32, ins: usize) -> usize {
	SAMPLED_BIT | ((ins.min(INS_MASK)) << INS_SHIFT) | ((stamp as usize) << STAMP_SHIFT)
}

#[inline]
fn is_sampled(c: usize) -> bool {
	c & SAMPLED_BIT != 0
}

#[inline]
fn stamp(c: usize) -> u32 {
	((c >> STAMP_SHIFT) & STAMP_MASK) as u32
}

#[inline]
fn probe_len(c: usize) -> usize {
	(c >> INS_SHIFT) & INS_MASK
}

/// MRU-sample period selected by PSEL; `None` means pure LIP (no sampling).
#[inline]
fn period_for(psel: u8) -> Option<u32> {
	match psel {
		0..=3 => None,
		4..=7 => Some(128),
		8..=11 => Some(32),
		_ => Some(16),
	}
}

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Per-bucket count of new-item admissions (drives sampling and tag stamps).
	miss_count: u32,
	/// Saturating selector in `0..=PSEL_MAX` choosing the MRU-sample period.
	psel: u8,
	/// Whether per-item tags may live in `lfu_counter` (ClassicLRU never uses it).
	tagging: bool,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let eviction_policy = settings.eviction_policy.build();
		let tagging = eviction_policy.name() == "ClassicLRU";
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy,
			miss_count: 0,
			psel: PSEL_MAX / 2,
			tagging,
		}
	}

	/// Current sample period (`None` = pure LIP).
	fn current_period(&self) -> Option<u32> {
		if self.tagging {
			period_for(self.psel)
		} else {
			Some(BIMODAL_PERIOD)
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

	/// Hit path (used by both `get` and `put` on an existing key). A sampled
	/// item hit after more admissions than its would-be probation depth is a
	/// hit cold insertion would have lost: reward PSEL. Either way the tag is
	/// cleared so each sample yields at most one signal. ClassicLRU's
	/// `get_from_position` does not touch `lfu_counter`, so no save/restore.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.tagging {
			let c = self.items[position].lfu_counter;
			if is_sampled(c) {
				let age = self.miss_count.wrapping_sub(stamp(c));
				if age as usize > probe_len(c) && self.psel < PSEL_MAX {
					self.psel += 1;
				}
				self.items[position].lfu_counter = 0;
			}
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	/// Adaptive bimodal LRU-insertion-point put: new items are inserted at the
	/// bucket's 25% position (from the LRU end), except every `period`-th new
	/// item (period chosen by PSEL), which goes to the MRU tail carrying a tag.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let mut evicted_item = if self.is_full() { self.evict_item() } else { None };
		if self.tagging {
			if let Some(victim) = evicted_item.as_mut() {
				if is_sampled(victim.lfu_counter) {
					// The sample used protected space for nothing.
					self.psel = self.psel.saturating_sub(1);
				}
				victim.lfu_counter = 0;
			}
		}
		self.miss_count = self.miss_count.wrapping_add(1);
		let ins = self.items.len() / INSERT_FRACTION_DIV;
		let sample = match self.current_period() {
			Some(p) => self.miss_count % p == 0,
			None => false,
		};
		let mut item = item;
		let idx = if sample {
			item.lfu_counter = if self.tagging { tag(self.miss_count, ins) } else { 0 };
			self.items.push(item);
			self.items.len() - 1
		} else {
			if self.tagging {
				item.lfu_counter = 0;
			}
			if ins == 0 {
				self.items.push(item);
				self.items.len() - 1
			} else {
				self.items.insert(ins, item);
				ins
			}
		};
		(&self.items[idx], evicted_item)
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;

	fn bucket(size: usize) -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::new(&AssociativeCacheSettings {
			num_of_buckets: 1,
			bucket_size: size,
			eviction_policy: EvictionPolicyKind::ClassicLRU,
		})
	}

	/// Fill a bucket so the next admission (miss_count -> 128) is sampled at
	/// the starting psel (7 → period 128). Returns the sampled key.
	fn setup_sampled(b: &mut AssociativeCacheBucket<i64, i64>) -> i64 {
		assert_eq!(b.psel, 7);
		for k in 0..127 {
			b.put(Item::new(k, k));
		}
		assert_eq!(b.miss_count, 127);
		let key = 1_000;
		let (inserted, _) = b.put(Item::new(key, 1));
		assert_eq!(inserted.key, key);
		assert!(is_sampled(inserted.lfu_counter));
		assert_eq!(b.items.last().unwrap().key, key);
		key
	}

	#[test]
	fn tag_round_trips() {
		let c = tag(0xDEAD_BEEF, 0x1_2345);
		assert!(is_sampled(c));
		assert_eq!(stamp(c), 0xDEAD_BEEF);
		assert_eq!(probe_len(c), 0xFFFF);
		assert!(!is_sampled(0));
	}

	#[test]
	fn late_hit_on_sample_raises_psel() {
		let mut b = bucket(512);
		let key = setup_sampled(&mut b);
		let ins = probe_len(b.items.last().unwrap().lfu_counter);
		for k in 0..(ins as i64 + 1) {
			b.put(Item::new(10_000 + k, k));
		}
		let before = b.psel;
		let hit = b.get(&key).unwrap();
		assert_eq!(hit.lfu_counter, 0);
		assert_eq!(b.psel, before + 1);
	}

	#[test]
	fn early_hit_on_sample_is_neutral_and_clears_tag() {
		let mut b = bucket(512);
		let key = setup_sampled(&mut b);
		b.put(Item::new(20_000, 0));
		let before = b.psel;
		let hit = b.get(&key).unwrap();
		assert_eq!(hit.lfu_counter, 0);
		assert_eq!(b.psel, before);
		// A second hit yields no further signal.
		b.get(&key);
		assert_eq!(b.psel, before);
	}

	#[test]
	fn unhit_sample_eviction_lowers_psel() {
		let mut b = bucket(128);
		let key = setup_sampled(&mut b);
		assert!(b.is_full());
		let before = b.psel;
		let mut lowered = false;
		for k in 0..2_000 {
			let (_, evicted) = b.put(Item::new(50_000 + k, k));
			if let Some(e) = evicted {
				assert_eq!(e.lfu_counter, 0);
				if e.key == key {
					lowered = true;
					break;
				}
			}
		}
		assert!(lowered);
		assert!(b.psel < before);
		assert!(b.psel <= PSEL_MAX);
		assert!(b.items.len() <= b.max_size);
	}

	#[test]
	fn non_lru_policy_does_not_tag() {
		let mut b: AssociativeCacheBucket<i64, i64> = AssociativeCacheBucket::new(&AssociativeCacheSettings {
			num_of_buckets: 1,
			bucket_size: 64,
			eviction_policy: EvictionPolicyKind::Lfu,
		});
		for k in 0..200 {
			b.put(Item::new(k, k));
		}
		assert!(b.items.iter().all(|i| i.lfu_counter == 0));
		assert!(b.items.len() <= 64);
	}
}
