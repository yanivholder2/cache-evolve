# Variant v003 Insight

## Helped

- hit-rate: Replacing recency-ordered victim choice with the lowest per-slot saturating access counter (incremented on get hits) keeps re-referenced keys in the bucket through bursts of one-shot inserts, which is the pollution pattern the hypothesis targeted on workload-a.

## Hurt

- per-op-time: `put` now scans every slot in a full bucket to find the minimum counter instead of using the baseline's direct choice, and `get` performs an extra counter write on each hit, so both op latencies moved in the wrong direction.
- metadata-size: The hypothesis expected one extra byte per slot, but the reported sizes for the item and cache types did not change, so the counter either fits in existing padding or is not captured by the size probe; the metadata dimension does not yet confirm the stated cost.

## Takeaway

A frequency signal does help hit-rate on workload-a, so frequency is worth pursuing as a separate axis from recency and geometry. The gain is paid for in per-op time from the full-bucket scan on `put`, so any follow-up should check whether the victim search can be cheaper (for example, tracking a minimum or using fewer ways) before stacking other changes on top.
