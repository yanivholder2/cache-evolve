# Variant v001 Insight

## Helped

- hit-rate: The `victim_freq == 0` early exit preserved victim identity exactly (older candidates visited first, replacement only on strictly-lower estimate), so hit-rate stayed bit-identical to the parent — the change's core neutrality claim held.

## Hurt

- per-op-time: Both `get` and `put` p50/p99 rose versus the parent, but the untouched `get` path moved by a similar margin as `put`, so the regression is run-to-run benchmark variance rather than a cost of the early exit — the predicted put-path saving never surfaced above that noise.

## Takeaway

Skipping the residual `sketch.estimate` calls past the count-min floor is genuinely hit-rate-neutral, but the work it removes — a few hash-plus-min lookups only when a freq-0 entry already sits near the LRU head — is too small to register against per-op-time measurement noise. The eviction walk is not actually the dominant put-path cost on this workload, so micro-trimming it yields no measurable speedup.
