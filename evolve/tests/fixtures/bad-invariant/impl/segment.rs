// Bad-Variant fixture: compiles, passes G2 round-trip, violates P3 slot_legality.
// MOCK_CARGO: g3_fail P3 slot_legality
//
// On construction, force-pushes a phantom item into bucket 0 whose key hashes to
// a different bucket. The legit Segment surface (`get`, `put`) still works, so G1
// and G2 pass; the G3 Invariant suite catches the illegal slot.
//
// `VariantSegment` is declared `<K, V>` to match the shape every Variant must
// expose (the shared g2_contract harness instantiates `VariantSegment::<i64, i64>`);
// only the `<i64, i64>` constructor is provided because the rejection-log
// machinery in scripts/gate-check.py is wired for that key/value type.
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
        let n = settings.num_of_normal_buckets;
        debug_assert!(
            n >= 3,
            "fixture requires num_of_normal_buckets >= 3 (need a non-edge bucket)"
        );
        let mut inner = PromotionColdSegment::<i64, i64>::new(settings);
        // Pick a key whose target bucket is neither 0 nor n-1: that excludes 0 from
        // both target and probing ((target+1) % n), guaranteeing slot illegality in
        // bucket 0 (independent of the concrete hash).
        let phantom_key = (0i64..(4 * n as i64))
            .find(|k| {
                let t = get_index(k, n);
                t != 0 && t != n - 1
            })
            .expect("no non-edge bucket found in search range");
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
