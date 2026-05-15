// Intentionally broken fixture: calls a non-existent method to trigger a G1 compile error.
use dash::dash::dash_settings::DashSettings;
use dash::dash::segments::{PromotionColdSegment, Segment, SegmentCore};
use dash::shared::item::Item;
use std::fmt::Debug;
use std::hash::Hash;

pub struct VariantSegment<K, V>
where
    K: Hash + Eq + Copy + Debug,
    V: Eq + Copy + Debug,
{
    inner: PromotionColdSegment<K, V>,
}

impl<K, V> VariantSegment<K, V>
where
    K: Hash + Eq + Copy + Debug,
    V: Eq + Copy + Debug,
{
    pub fn new(settings: &DashSettings) -> Self {
        Self {
            inner: PromotionColdSegment::new(settings),
        }
    }
}

impl<K, V> Segment<K, V> for VariantSegment<K, V>
where
    K: Hash + Eq + Copy + Debug,
    V: Eq + Copy + Debug,
{
    fn core(&self) -> &SegmentCore<K, V> {
        self.inner.core()
    }

    fn core_mut(&mut self) -> &mut SegmentCore<K, V> {
        self.inner.core_mut()
    }

    fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
        // G1 failure: `private_lookup` is not a method on PromotionColdSegment
        self.inner.private_lookup(key)
    }
}
