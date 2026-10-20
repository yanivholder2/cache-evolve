# Variant v002 Insight

## Helped

- hit-rate: `DashSegment::get` now calls `promotion_get(..., InsertPosition::Back)` instead of `Front`, so an item promoted from the stash lands at the MRU end of its target bucket and is no longer the first eviction candidate on the next insert into that bucket. On the LIRS workload-a trace this keeps re-referenced stash items resident and is the only measured dimension for this variant.

## Hurt

## Takeaway

Changing one argument in `DashSegment::get` moves the stash-hit landing slot from LRU end to MRU end, and the measured gain on the LIRS trace supports the hypothesis that re-referenced stash items were being evicted too quickly. The source comment on `promotion_get` still describes Front as the default and the `get` docstring calls LRU-end promotion "the baseline design", so the Front/Back rationale is documented inconsistently in this variant. The variant's fitness vector reports only hit-rate, so this insight makes no claim about metadata-size or per-op-time.
