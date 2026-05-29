// External fixture proving `Segment` is a genuine Level 3 extension point.
//
// `MinimalSegment` lives entirely outside `src/` and is compiled against the
// crate's public API only. It wraps `PromotionColdSegment` as a minimal but
// valid delegation: every implementation axiom (put→stash, get→promotion) is
// exercised through the trait surface with no access to crate internals.
use dash::dash::dash_settings::DashSettings;
use dash::dash::segments::{PromotionColdSegment, Segment, SegmentCore};
use dash::dash::Dash;
use dash::shared::eviction_policy::EvictionPolicyKind;
use dash::shared::item::Item;
use std::fmt::Debug;
use std::hash::Hash;

// ---------------------------------------------------------------------------
// MinimalSegment: the external Segment implementation under test
// ---------------------------------------------------------------------------

struct MinimalSegment<K, V>
where
    K: Hash + Eq + Copy + Debug + 'static,
    V: Eq + Copy + Debug + 'static,
{
    inner: PromotionColdSegment<K, V>,
}

impl<K, V> MinimalSegment<K, V>
where
    K: Hash + Eq + Copy + Debug + 'static,
    V: Eq + Copy + Debug + 'static,
{
    fn new(settings: &DashSettings) -> Self {
        Self {
            inner: PromotionColdSegment::new(settings),
        }
    }
}

impl<K, V> Segment<K, V> for MinimalSegment<K, V>
where
    K: Hash + Eq + Copy + Debug + 'static,
    V: Eq + Copy + Debug + 'static,
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

// ---------------------------------------------------------------------------
// Shared test helper
// ---------------------------------------------------------------------------

fn minimal_settings() -> DashSettings {
    DashSettings {
        num_of_segments: 1,
        num_of_normal_buckets: 4,
        num_of_stash_buckets: 2,
        bucket_size: 2,
        eviction_policy: EvictionPolicyKind::ClassicLRU,
        debug_mode: 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// A new item survives a put→get round trip: proves `put` (default impl) routes
/// the item somewhere the segment's `get` can find it.
#[test]
fn put_then_get_round_trip() {
    let mut seg = MinimalSegment::<i64, i64>::new(&minimal_settings());
    seg.put(Item::new(42, 100));
    let item = seg.get(&42).expect("item must be resident after put");
    assert_eq!(item.value, 100);
}

/// A key that was never put must not be returned by `get`.
#[test]
fn get_absent_key_returns_none() {
    let mut seg = MinimalSegment::<i64, i64>::new(&minimal_settings());
    assert!(seg.get(&99).is_none());
}

/// Multiple distinct keys are all independently retrievable through the external
/// impl — proves routing is key-specific, not global.
#[test]
fn multiple_keys_all_retrievable() {
    let settings = minimal_settings();
    // Keys chosen to land in different target buckets so no same-bucket eviction can occur.
    let n = settings.num_of_normal_buckets;
    let key_a = (0_i64..).find(|&k| dash::get_index(&k, n) == 0).unwrap();
    let key_b = (0_i64..).find(|&k| dash::get_index(&k, n) == 1).unwrap();

    let mut seg = MinimalSegment::<i64, i64>::new(&settings);
    seg.put(Item::new(key_a, 10));
    seg.put(Item::new(key_b, 20));
    assert_eq!(seg.get(&key_a).unwrap().value, 10);
    assert_eq!(seg.get(&key_b).unwrap().value, 20);
}

/// Confirms the external type compiles against `Dash<K, V, S: Segment<K, V>>`,
/// the generic shell that Level 3 Variants must fit into.
#[test]
fn external_segment_works_inside_dash() {
    let settings = minimal_settings();
    let mut cache: Dash<i64, i64, MinimalSegment<i64, i64>> =
        Dash::new(1, || MinimalSegment::new(&settings));
    cache.put(7, 70);
    assert_eq!(*cache.get_and_update_item(&7).unwrap(), 70);
}

/// Re-inserting a resident key must panic in debug builds — enforces the
/// uniqueness invariant (ADR 0004) via the `debug_assert` in the default `put`.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "already present")]
fn put_duplicate_key_panics_in_debug() {
    let mut seg = MinimalSegment::<i64, i64>::new(&minimal_settings());
    seg.put(Item::new(42, 100));
    seg.put(Item::new(42, 200));
}
