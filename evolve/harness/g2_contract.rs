// Copied into $WORK_DIR/tests/g2_contract.rs at build time — not compiled from here.
// Kept as a real .rs file so rustfmt, IDEs, and trait-drift review can target it.
use dash::dash::dash_settings::DashSettings;
use dash::dash::segments::Segment;
use dash::shared::eviction_policy::EvictionPolicy;
use dash::shared::item::Item;
use variant_check::VariantSegment;

fn default_settings() -> DashSettings {
    DashSettings {
        num_of_segments: 1,
        num_of_normal_buckets: 4,
        num_of_stash_buckets: 2,
        bucket_size: 2,
        eviction_policy: EvictionPolicy::ClassicLRU,
        debug_mode: 0,
    }
}

#[test]
fn g2_constructor_succeeds() {
    let _seg = VariantSegment::<i64, i64>::new(&default_settings());
}

#[test]
fn g2_put_get_round_trip() {
    let mut seg = VariantSegment::<i64, i64>::new(&default_settings());
    seg.put(Item::new(1i64, 100i64));
    assert_eq!(seg.get(&1i64).expect("item must be resident after put").value, 100i64);
}

#[test]
fn g2_core_accessible() {
    let seg = VariantSegment::<i64, i64>::new(&default_settings());
    let _ = seg.core();
}

#[test]
fn g2_valid_eviction_policy_ordinal() {
    if let Ok(manifest) = std::fs::read_to_string("manifest.toml") {
        for line in manifest.lines() {
            if line.trim().starts_with("eviction_policy_ordinal") {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    let ordinal_str = parts[1].trim();
                    if let Ok(ordinal) = ordinal_str.parse::<usize>() {
                        assert!(
                            EvictionPolicy::from_usize(ordinal).is_some(),
                            "eviction_policy_ordinal={} is not a valid EvictionPolicy index",
                            ordinal
                        );
                    }
                }
            }
        }
    }
}
