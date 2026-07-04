# Variant v003 Insight

## Helped

- hit-rate: Admitting new keys at index `0` (the ClassicLRU victim end) makes one-hit-wonders and scan traffic the immediate next eviction, so the established working set at the back is no longer pushed out by transient keys — the scan-resistance the hypothesis predicted, and hit-rate rose over baseline.

## Hurt

- (none measured): The variant's fitness vector only reports `hit-rate`; `metadata-size` and `per-op-time` were not re-measured, so any cost of the new front-`insert(0, …)` shift versus the baseline back-push is not captured here.

## Takeaway

Changing only the *insertion position* — admitting fresh keys as front "probationary" victims while hits still promote toward the back via the unchanged policy — is enough to add second-chance scan resistance to ClassicLRU without touching the eviction rule, bucket geometry, or routing. The working-set protection alone lifted hit rate on the scan-heavy trace.
