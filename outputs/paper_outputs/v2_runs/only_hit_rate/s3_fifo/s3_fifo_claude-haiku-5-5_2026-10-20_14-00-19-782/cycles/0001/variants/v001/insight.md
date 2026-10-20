# Variant v001 Insight

## Helped

(none: no measured dimension improved)

## Hurt

- hit-rate: Lowering the small-queue promotion bar from two hits to one (`tail.freq > 1` to `tail.freq > 0` in `evict_small()`) lets one-hit admission entries displace residents in the main queue, and on this trace that displacement costs more hits than the extra promotions recover.

## Takeaway

The two-hit promotion bar in the baseline is doing useful filtering: admitting every reused small-queue entry into main pushes out entries that were already earning their place there, so the change is a net loss on lirs workload-a at size 2048. The change touched only the promotion predicate, so any effect comes from main-queue churn rather than from metadata or the ghost path. Only one trace and one capacity were measured, and the variant's fitness vector reports hit-rate alone, so metadata-size and per-op-time are not assessed here.
