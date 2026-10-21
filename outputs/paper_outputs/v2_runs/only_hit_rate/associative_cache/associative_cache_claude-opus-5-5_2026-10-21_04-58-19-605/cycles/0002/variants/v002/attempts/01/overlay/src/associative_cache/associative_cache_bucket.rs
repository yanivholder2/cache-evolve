use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Insertion depth for new (missed) keys under ClassicLRU, as a fraction of the
/// bucket length measured from the eviction end (index 0). 1/2 = midpoint.
const INSERT_DEPTH_NUM: usize = 1;
const INSERT_DEPTH_DEN: usize = 2;

/// Number of recently evicted key fingerprints remembered per bucket.
const GHOST_SIZE: usize = 16;

/// Sentinel marking an empty / consumed ghost slot. Never produced by `fingerprint`.
const GHOST_EMPTY: u64 = 0;

/// Salt hashed ahead of the key so fingerprints differ from the bucket router's hash.
const FINGERPRINT_SALT: u64 = 0x9E37_79B9_7F4A_7C15;

#[inline]
fn fingerprint<K: Hash>(key: &K) -> u64 {
	let mut hasher = DefaultHasher::new();
	FINGERPRINT_SALT.hash(&mut hasher);
	key.hash(&mut hasher);
	let fp = hasher.finish();
	if fp == GHOST_EMPTY { 1 } else { fp }
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
	/// When true, new keys are inserted at the bucket midpoint instead of the
	/// MRU tail (LIP/BIP-style). Only enabled for ClassicLRU.
	midpoint_insert: bool,
	/// Ring of fingerprints of recently evicted keys (used only with midpoint_insert).
	ghost: Vec<u64>,
	/// Next ring slot to overwrite once `ghost` is full.
	ghost_head: usize,
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
			midpoint_insert: settings.eviction_policy == EvictionPolicyKind::ClassicLRU,
			ghost: Vec::with_capacity(GHOST_SIZE),
			ghost_head: 0,
		}
	}

	#[inline]
	fn ghost_record(&mut self, fp: u64) {
		if self.ghost.len() < GHOST_SIZE {
			self.ghost.push(fp);
		} else {
			self.ghost[self.ghost_head] = fp;
			self.ghost_head = (self.ghost_head + 1) % GHOST_SIZE;
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

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing key: baseline behaviour (update value, promote per policy).
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		if !self.midpoint_insert {
			// Baseline miss path: capacity check, then push to the tail.
			let evicted_item = if self.is_full() { self.evict_item() } else { None };
			self.items.push(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, evicted_item);
		}

		// Ghost lookup before evicting.
		let fp = fingerprint(&item.key);
		let in_ghost = self.ghost.iter().position(|&g| g == fp);
		// Consume the ghost entry first so a single ghost hit promotes only once
		// (and so the ring write below can't be clobbered by the sentinel).
		if let Some(g) = in_ghost {
			self.ghost[g] = GHOST_EMPTY;
		}

		// Capacity check runs before the insert, so size never exceeds max_size.
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		if let Some(ev) = evicted_item.as_ref() {
			let efp = fingerprint(&ev.key);
			self.ghost_record(efp);
		}

		if in_ghost.is_some() {
			// Recently evicted key returning: grant MRU protection.
			self.items.push(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, evicted_item);
		}

		// Midpoint insertion: computed after the eviction.
		let idx = self.items.len() * INSERT_DEPTH_NUM / INSERT_DEPTH_DEN;
		self.items.insert(idx, item);
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
