# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The new `evict_item` override zeroes the whole `reuse_mask` on a fully-protected eviction, demoting the entire resident hot set at once; keys that had legitimately earned protection lost immunity and were evicted by subsequent scan/cold traffic before they could re-earn it, so hit-rate fell below the parent's permanent-protection ceiling.
- per-op-time: The extra saturated-regime bookkeeping on the eviction path (`live` recompute, `fully_protected` compare, and mask-clear inside `evict_item`) lengthened the `get`/`put` p99 tail (p50 summary held at 100, but p99 doubled), a cost the parent's untouched eviction path did not pay.

## Takeaway

The bet that a fully-saturated bucket is a regime where the reuse bit "has lost all discriminating power" is false: even when every bit is set, the mask still preserves per-key reuse evidence that outlives one eviction, and clearing it globally throws that away and readmits stale/scan traffic. Permanent one-hit protection is load-bearing even at saturation — no hit-rate headroom survives above the firm ceiling by re-opening protected slots.
