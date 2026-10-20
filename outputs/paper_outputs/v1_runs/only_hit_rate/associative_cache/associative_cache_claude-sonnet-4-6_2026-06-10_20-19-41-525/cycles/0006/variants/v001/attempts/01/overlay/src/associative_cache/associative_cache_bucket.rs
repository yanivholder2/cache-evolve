//! AssociativeCacheBucket with mid-point insertion and CLOCK eviction.
//!
//! **v001 (cycle/0006) hypothesis**: replacing cold-first-scan + bulk-decay
//! with a persistent CLOCK-hand eviction distributes aging pressure
//! proportionally across all warm items rather than evicting whichever cold
//! item happens to appear first in index order, improving hit-rate.
//!
//! Inherited from cycle/0005/v003:
//! - Wide-bucket geometry (factor-4 repartition in `geometry.rs`): unchanged.
//! - Counter increment capped at 4 in `ClassicLRU::get_from_position`
//!   (`eviction_policy.rs`): unchanged.
//! - Mid-point insertion: new items are inserted at position `len / 2`.
//!
//! New in this variant:
//! - CLOCK eviction: a persistent `clock_hand` sweeps through items giving each
//!   warm item a per-step decrement opportunity rather than evicting the first
//!   cold item in index order and decaying all items in bulk.

use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
    K: Hash + Eq + Copy + Debug,
    V: Eq + Copy + Debug,
{
    items: Vec<Item<K, V>>,
    max_size: usize,
    eviction_policy: Box<dyn EvictionPolicy<K, V>>,
    clock_hand: usize,
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
            clock_hand: 0,
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

    /// Evicts an item using CLOCK semantics.
    ///
    /// The clock hand sweeps through items in order. At each position:
    /// - If `lfu_counter == 0`: the item is cold — evict it and advance the
    ///   hand past the gap.
    /// - Otherwise: decrement its counter (second-chance), advance the hand.
    ///
    /// The loop runs for at most `n * 5` iterations, which is sufficient since
    /// `ClassicLRU` caps counters at 4: any item becomes cold within 4 passes.
    ///
    /// Returns `None` only when the bucket is empty.
    fn evict_item(&mut self) -> Option<Item<K, V>> {
        if self.items.is_empty() {
            return None;
        }
        let n = self.items.len();
        // CLOCK sweep: at most n full rotations (cap==4 guarantees termination).
        for _ in 0..n * 5 {
            let hand = self.clock_hand % self.items.len();
            if self.items[hand].lfu_counter == 0 {
                // Found a cold item — evict it and advance hand past the gap.
                let evicted = self.items.remove(hand);
                self.clock_hand = if self.items.is_empty() { 0 } else { hand % self.items.len() };
                return Some(evicted);
            }
            // Give this item a second chance: decrement its counter and move on.
            self.items[hand].lfu_counter -= 1;
            self.clock_hand = (hand + 1) % self.items.len();
        }
        // Safety fallback (should not be reached with cap==4): evict hand position.
        let hand = self.clock_hand % self.items.len();
        let evicted = self.items.remove(hand);
        self.clock_hand = if self.items.is_empty() { 0 } else { hand % self.items.len() };
        Some(evicted)
    }

    /// Puts an item into the bucket using mid-point insertion.
    ///
    /// - Existing-key update: update value in place, call `get_from_position`
    ///   per policy, return.
    /// - New key, bucket full: evict via CLOCK `evict_item` first, then insert
    ///   at midpoint.
    /// - New key, bucket not full: insert at midpoint directly.
    ///
    /// After insertion the clock hand is adjusted: inserting at `mid` shifts
    /// all elements at positions >= mid rightward by one, so if
    /// `clock_hand >= mid` we increment the hand to keep it pointing at the
    /// same logical item.
    fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
        if let Some(position) = self.get_position(&item.key) {
            self.items[position].value = item.value;
            let pushed_item = self.get_from_position(position);
            return (pushed_item, None);
        }
        let evicted_item = if self.is_full() { self.evict_item() } else { None };
        let mid = self.items.len() / 2;
        self.items.insert(mid, item);
        // Adjust hand: insertion at mid shifts all items at >= mid rightward.
        if mid <= self.clock_hand && self.clock_hand + 1 < self.items.len() {
            self.clock_hand += 1;
        }
        (&self.items[mid], evicted_item)
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
