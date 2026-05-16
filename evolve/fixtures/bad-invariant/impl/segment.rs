// Bad-Variant fixture: compiles, passes G2 round-trip, violates P3 slot_legality.
//
// On construction, force-pushes a phantom item into bucket 0 whose key hashes to
// a different bucket. The legit Segment surface (`get`, `put`) still works, so G1
// and G2 pass; the G3 Invariant suite catches the illegal slot.
use dash::dash::dash_settings::DashSettings;
use dash::dash::segments::{PromotionColdSegment, Segment, SegmentCore};
use dash::get_index;
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

impl VariantSegment<i64, i64> {
    pub fn new(settings: &DashSettings) -> Self {
        let mut inner = PromotionColdSegment::<i64, i64>::new(settings);
        // Pick a key whose target bucket is neither 0 nor n-1: that excludes 0 from
        // both target and probing ((target+1) % n), guaranteeing slot illegality in
        // bucket 0 (independent of the concrete hash).
        let n = settings.num_of_normal_buckets;
        let phantom_key = (0i64..1000)
            .find(|k| {
                let t = get_index(k, n);
                t != 0 && t != n - 1
            })
            .expect("no key in 0..1000 hashes to a non-edge bucket");
        inner
            .core_mut()
            .force_push_normal_bucket(0, Item::new(phantom_key, phantom_key));
        Self { inner }
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
        self.inner.get(key)
    }
}
