# Variant v001 Insight

## Helped

- None attributable to the change. The per-op-time timings moved, but the diff only touches the promotion test in `evict_small()`, and `get()` and `put()`'s hot path are unchanged, so the latency shift is more likely measurement noise or environment than this edit.

## Hurt

- hit-rate: Promoting a small-queue entry after one hit instead of two moves singly-reused keys into main earlier, which displaces main-queue residents that were holding useful frequency state, so workload-a at capacity 2048 loses a little hit rate.

## Takeaway

On workload-a at capacity 2048, the single-line change from `freq > 1` to `freq > 0` in `evict_small()` does not help: admitting one-reuse keys to main displaces more useful residents than it saves, and the hit-rate loss is the only effect the diff can explain. Metadata size is unchanged, as expected for a pure control-flow edit, and any per-op timing difference should be re-measured before it is credited to the promotion threshold.
