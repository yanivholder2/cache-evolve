# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Promoting a small-queue key to main the instant its `get()` bump reaches `freq >= 2`, rather than lazily at `evict_small()` time, left hit-rate unchanged from the parent on this LIRS trace. The change only alters *when* a multi-hit key moves to main, not *which* keys end up there: with `small_size` at capacity/24 (~85 slots), the admission queue does not churn fast enough to drop a freshly re-hit key to ghost before its accumulated `freq` is read at eviction, so eager and lazy promotion select the identical main-resident set and produce the identical hit sequence. The hypothesis's premise — that in-residence reuse signals were being lost before eviction read them — does not hold for this workload at this size; promotion *timing* is not a live lever here.
