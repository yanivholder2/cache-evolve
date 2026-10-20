# Variant v002 Insight

## Helped

- (none)

## Hurt

- per-op-time: The hit path in `get` and the existing-key branch of `put` now read `prev` and branch before setting REFERENCED, replacing the unconditional OR from the parent. On this workload the extra load-and-branch cost more than the skipped store saved, so both operations slowed, and `get` p50 and p99 rose more than `put` in relative terms.

## Takeaway

Skipping an already-set reference bit does not pay off on the packed `prev` word: the parent's unconditional OR is a single read-modify-write to a line the code already touches, so the guard adds a branch without removing meaningful work. The hit-rate and metadata-size dimensions are unchanged, as the policy and layout are untouched, so the entire effect is on latency. The lesson is that the reference-bit write should stay unconditional on the hit path.
