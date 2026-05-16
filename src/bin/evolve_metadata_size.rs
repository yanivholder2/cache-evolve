//! Static metadata-size measurement for fitness plugin consumption.
//!
//! Emits one plugin-shape JSON record:
//! `{ "dim": "metadata-size", "values": [{"type","bytes"},...], "summary": <Item bytes>, "higher_is_better": false }`.
//!
//! `summary` is `mem::size_of::<Item<u64,u64>>()` — the per-entry overhead that scales with cache population.
use dash::associative_cache::AssociativeCache;
use dash::dash::segments::PromotionColdSegment;
use dash::dash::Dash;
use dash::fitness_json::write_plugin_json;
use dash::shared::item::Item;
use std::mem;

fn main() {
    let entries = [
        ("Item<u64,u64>", mem::size_of::<Item<u64, u64>>()),
        ("AssociativeCache<u64,u64>", mem::size_of::<AssociativeCache<u64, u64>>()),
        ("Dash<u64,u64,PromotionColdSegment>", mem::size_of::<Dash<u64, u64, PromotionColdSegment<u64, u64>>>()),
    ];

    let values_inner: Vec<String> = entries
        .iter()
        .map(|(name, bytes)| format!(r#"{{"type":"{name}","bytes":{bytes}}}"#))
        .collect();
    let values = format!("[{}]", values_inner.join(","));
    let summary = mem::size_of::<Item<u64, u64>>();

    println!("{}", write_plugin_json("metadata-size", &values, summary, false));
}
