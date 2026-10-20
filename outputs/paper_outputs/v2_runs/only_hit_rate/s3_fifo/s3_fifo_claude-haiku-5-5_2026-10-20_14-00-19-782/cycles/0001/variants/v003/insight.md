# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Shrinking the small admission queue from a quarter to a tenth of capacity left hit-rate on the lirs workload-a trace at size 2048 exactly where the parent put it, so the hypothesis's premise that the baseline over-weights admission did not hold at this size. The change is a single constant in `try_new()`, so the null result points at the partition ratio itself rather than at any side effect of the eviction logic. One trace at one size cannot show whether the ratio matters elsewhere, and the variant's fitness vector reports only hit-rate, so the metadata-size and per-op-time effects are unmeasured here.
