/// Report mem::size_of for the key cache types as JSON.
///
/// Output shape matches fitness plugin contract:
/// { dim, values: [{type, bytes}], summary, higher_is_better }
use dash::associative_cache::AssociativeCache;
use dash::dash::Dash;
use dash::dash::segments::PromotionColdSegment;
use dash::shared::item::Item;
use std::mem;

fn main() {
    let entries = [
        ("Item<u64,u64>", mem::size_of::<Item<u64, u64>>()),
        ("AssociativeCache<u64,u64>", mem::size_of::<AssociativeCache<u64, u64>>()),
        ("Dash<u64,u64,PromotionColdSegment>", mem::size_of::<Dash<u64, u64, PromotionColdSegment<u64, u64>>>()),
    ];

    let values: Vec<String> = entries
        .iter()
        .map(|(name, bytes)| format!(r#"{{"type":"{name}","bytes":{bytes}}}"#))
        .collect();

    // summary = Item size (the per-entry overhead that scales with cache population)
    let summary = mem::size_of::<Item<u64, u64>>();

    println!(
        r#"{{"dim":"metadata-size","values":[{values}],"summary":{summary},"higher_is_better":false}}"#,
        values = values.join(","),
    );
}
