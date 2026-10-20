# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Routing ghost-hit re-admissions through `small` instead of `main` left hit-rate on the lirs workload-a trace at size 2048 unchanged from the parent. The code does what H02 describes: a re-admitted key sits in `small` with `freq = 0`, and `evict_small()` only promotes at `freq > 1`, so it now needs two `get()` hits (not one, as H02 states) before reaching `main`. The lack of any measurable movement suggests that on this trace ghost keys are rarely re-referenced before falling out of `small`, so the ghost-admission axis is inert here. The fitness vector also omits metadata-size and per-op-time, so this variant's effect on those dimensions cannot be assessed from the evidence present.
