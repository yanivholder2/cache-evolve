# Variant v003 Insight

## Helped

- per-op-time (`get`, the summary): Not caused by H03. `get` is byte-identical to the parent's, and the only new code is one `u8` increment and one compare on the graduation path inside `evict_small_inner`, which `get` never reaches. This improvement mirrors the parent's unexplained `get` regression, especially at p99, so both are most likely noise or code layout. It does not show any real effect of the damping counter. `put` held flat, which fits a single extra branch on an already rare path.

## Hurt

- metadata-size (`S3Fifo<u64,u64>` row only; the `Entry` summary did not move): The new release-build `grow_credit: u8` field sits after the `usize` fields, so it adds one byte and then gets padded to a full 8-byte word in the struct. This is a fixed per-cache cost, not a per-entry cost.

(hit-rate did not move at the measured precision.)

## Takeaway

Halving the grow-small trigger with `GROW_SMALL_EVERY = 2` had no effect on workload-a's hit-rate. The graduate is still pushed into main through the same `evict_main` path, and the change only alters how often the main→small target transfer happens. On this trace, the split's balance point is evidently not limited by grow-small pressure. Either the ghost-hit shrink side dominates no matter how fast grow-small acts, or a shifted split does not change which keys stay resident. So `GROW_SMALL_EVERY` is not a useful lever here, and H03's "bigger main helps" theory should not be pursued through this trigger. Before trying higher values, use `h03_split_report`'s `grow_events`/`grows`/`settled_small` to check whether the settled split actually moved.
