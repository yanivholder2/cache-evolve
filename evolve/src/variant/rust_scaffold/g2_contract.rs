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
fn g2_valid_eviction_policy_name() {
    let manifest = std::fs::read_to_string("manifest.toml")
        .expect("manifest.toml must be readable from the workspace root");
    let mut found = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("eviction_policy") {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let raw = parts[1].trim();
                let name = raw.trim_matches(|c: char| c == '"' || c == '\'');
                assert!(
                    EvictionPolicy::from_name(name).is_some(),
                    "eviction_policy={:?} is not a valid EvictionPolicy name",
                    name
                );
                found = true;
            }
        }
    }
    assert!(
        found,
        "manifest.toml has no eviction_policy = ... line"
    );
}
