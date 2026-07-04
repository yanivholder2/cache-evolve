# Variant v003 Insight

## Helped

- hit-rate: No improvement — the predicted geometry×promotion synergy did not materialize; widening the buckets failed to rehabilitate the two-hit threshold.

## Hurt

- hit-rate: Regressed below the parent. The parent promotes on the *first* hit over baseline (narrow) geometry; this variant simultaneously doubled bucket width and raised the protection bar to a *second* hit (`lfu_counter >= 2`), so more keys sit probationary in `eviction_candidate_index`'s scan and are evicted before earning protection — the wider, longer probation lap was not enough to let them collect that second access, and the larger collision set diluted hits further.

## Takeaway

Reviving the two-hit promotion threshold inside a wider bucket did not rescue it: pairing the one untested geometry change with the known-bad promotion lever still lands below the parent's single-hit SLRU. This isolates the answer the hypothesis sought — the two-hit regression is intrinsic to demanding a second proof of reuse on this workload, not an artifact of the narrow baseline bucket; longer probation laps do not buy back the keys that a stricter threshold strands.
