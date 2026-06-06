# Variant v001 Insight

## Helped

- (none): Conservative update sharpened the frequency estimate in principle, but on the wide factor-16 sketch hit-rate stayed flat within noise — the wide table's collision over-count was already low enough that minimal-increment had no aliasing to correct.

## Hurt

- per-op-time: The new first pass in `increment` scans all `depth` rows to find the per-key minimum before raising any counter, doubling the sketch touches on the write hot path; both `get` and `put` p50/p99 rose with no hit-rate return to justify the tax.

## Takeaway

This variant answers the standing question carried since Cycle 0006: conservative update's hit-rate gain on the *narrow* factor-4 sketch does **not** transfer to the wide factor-16 base. Where the wide geometry has already suppressed counter aliasing, minimal-increment buys no accuracy but still pays the full per-op min-scan cost — so on this base it is a strict regression, and estimate-sharpening via conservative counting is ruled flat here.
