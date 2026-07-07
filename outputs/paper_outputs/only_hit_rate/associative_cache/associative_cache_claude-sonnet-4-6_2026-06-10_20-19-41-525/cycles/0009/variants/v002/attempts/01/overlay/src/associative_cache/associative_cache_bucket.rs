use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// SLRU bucket: physically partitions `items` into two segments.
///
/// Layout: `items[0..prob_count]` is the probationary segment (index 0 = oldest),
///         `items[prob_count..]` is the protected segment (index prob_count = oldest).
///
/// Fixed caps: `prob_cap = max_size / 2`, `prot_cap = max_size - prob_cap`.
///
/// New items always enter the back of the probationary segment.  A cache hit on
/// a probationary item promotes it to the back of the protected segment (the
/// oldest protected item is demoted to probationary-front if protected is full).
/// A cache hit on a protected item moves it to the back of the protected segment.
/// Eviction always removes the oldest probationary item first; if probationary is
/// empty, it removes the oldest protected item.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
    K: Hash + Eq + Copy + Debug,
    V: Eq + Copy + Debug,
{
    items: Vec<Item<K, V>>,
    max_size: usize,
    eviction_policy: Box<dyn EvictionPolicy<K, V>>,
    /// Live count of probationary items occupying `items[0..prob_count]`.
    prob_count: usize,
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
            prob_count: 0,
        }
    }

    /// Maximum number of items allowed in the protected segment.
    #[inline]
    fn prot_cap(&self) -> usize {
        let prob_cap = self.max_size / 2;
        self.max_size - prob_cap
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

    /// SLRU `put`: existing keys are updated in place (then promoted per
    /// `get_from_position`); new keys are inserted at the back of the
    /// probationary segment (`items[prob_count]`).
    fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
        // Existing-key update: value in place, then apply hit logic.
        if let Some(position) = self.get_position(&item.key) {
            self.get_items_mut()[position].value = item.value;
            let pushed_item = self.get_from_position(position);
            return (pushed_item, None);
        }

        // New key: evict if full, then insert at back of probationary.
        let evicted_item = if self.is_full() { self.evict_item() } else { None };

        // Insert at the back of the probationary segment; items[prob_count..]
        // (protected segment) shift right by one, keeping their relative order.
        self.items.insert(self.prob_count, item);
        self.prob_count += 1;

        // The newly inserted item now sits at index prob_count - 1.
        let inserted = &self.items[self.prob_count - 1];
        (inserted, evicted_item)
    }

    /// SLRU eviction: remove the oldest probationary item (items[0]) if any,
    /// otherwise remove the oldest protected item (also items[0] when
    /// prob_count == 0).
    fn evict_item(&mut self) -> Option<Item<K, V>> {
        if self.items.is_empty() {
            return None;
        }
        // items[0] is always the oldest item in whichever segment is non-empty first
        // (probationary takes priority since it occupies [0..prob_count]).
        let evicted = self.items.remove(0);
        if self.prob_count > 0 {
            self.prob_count -= 1;
        }
        Some(evicted)
    }

    /// SLRU access: increment the lfu_counter (for inspection) and apply the
    /// segment-appropriate promotion/recency update.
    ///
    /// Probationary hit (`position < prob_count`):
    ///   Remove from probationary; if protected is now full, demote its oldest
    ///   item to probationary-front; push the accessed item to protected-back.
    ///
    /// Protected hit (`position >= prob_count`):
    ///   Remove from its current position and push to protected-back.
    ///
    /// Returns a reference to the item at its new position (always the last element).
    fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
        // Increment lfu_counter for counter tracking (not used for eviction here).
        self.items[position].lfu_counter += 1;

        if position < self.prob_count {
            // ---- Probationary hit ----
            let item = self.items.remove(position);
            self.prob_count -= 1;

            // Check whether the protected segment is full after the removal.
            // Protected count = total items - probationary items.
            let prot_count = self.items.len() - self.prob_count;
            if prot_count == self.prot_cap() {
                // Protected is full: demote its oldest item (at index prob_count,
                // the first entry in the protected sub-slice) to the front of
                // probationary (index 0, the oldest probationary slot).
                let demoted = self.items.remove(self.prob_count);
                self.items.insert(0, demoted);
                self.prob_count += 1;
            }

            // Promote accessed item to the back of the protected segment.
            self.items.push(item);
        } else {
            // ---- Protected hit ----
            // Move to back of protected (most-recently-used position).
            let item = self.items.remove(position);
            self.items.push(item);
        }

        self.items.last().expect("items must be non-empty after a hit")
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
