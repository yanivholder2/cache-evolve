# Variant v001 Insight

## Helped

- metadata-size: Reusing the existing `Item.lfu_counter` field as a "seen" flag kept `Item<u64,u64>` at the same width as baseline — the second-hit gate added behaviour without growing per-item state.

## Hurt

- hit-rate: Gating move-to-back on a second hit pinned freshly-inserted items at the bucket's eviction front (index 0, `lfu_counter = 0`) so a genuinely-hot newcomer was evicted on the very next miss before it could earn its second touch, costing hits on workload-a.

## Takeaway

On workload-a's bucket-local access pattern, ClassicLRU's blanket promotion is not actually being abused by scans — the bucket is too narrow for a scan to sweep hot items down before they re-hit. Deferring promotion instead penalises real reuse during an item's vulnerable first-hit window, suggesting future variants should protect newly-touched items rather than withhold promotion from them.
