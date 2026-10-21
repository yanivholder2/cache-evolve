# Variant v001 Insight

## Helped

- (none): No dimension improved; the looser promotion gate did not pay off.

## Hurt

- hit-rate: Lowering the `evict_small` gate from `freq > 1` to `freq >= 1` lets entries with a single reuse into main, so main takes in more marginal items and displaces more useful residents. The result is slightly worse than the parent on the LIRS trace.
- metadata-size / per-op-time: These dimensions are missing from the fitness vector, so v001 can't be compared with the parent on them. The change adds no fields and no work on `get` or `put`, so they were probably unaffected.

## Takeaway

On this LIRS workload the strict two-hit filter in the small queue was doing useful work. Admitting every entry that was reused once into main only added pollution, and ghost-based re-admission already recovers genuinely hot keys. A one-hit gate is not a cheap win here. Further gains probably need a different lever, such as main-queue eviction, the small/main split, or ghost sizing, rather than a looser threshold.
