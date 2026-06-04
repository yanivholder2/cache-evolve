# Variant v002 Insight

## Helped

_(no dimension improved)_

## Hurt

_(no dimension degraded)_

## Takeaway

Removing `timestamp: Instant` from `Item` did not disturb hit-rate, confirming the field was truly dead weight for the ClassicLRU evaluation path — the policy never reads it, so eviction decisions and cache behaviour were unaffected. The `metadata-size` dimension does not appear in this variant's fitness vector at all, which means the predicted 40% per-item reduction (40 → 24 bytes) was not captured by the evaluator and cannot be confirmed from the measured output. The structural change is correct and the code compiles, but the key claim of this hypothesis remains unverified until metadata-size measurement is included in the evaluation run.
