# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Replacing the LRU-front fallback with `min_by_key(lfu_counter)` marginally reduced hit-rate, suggesting that when all bucket residents have non-zero counters, the LRU-front item (index 0) is already the correct eviction victim — its recency staleness is the reliable signal — and overriding it with a frequency minimum can select a recently-inserted item whose counter is low simply because it is new, not because it is cold under the true Zipfian access distribution.

## Takeaway

The accumulated `lfu_counter` values in `ClassicLRUFreqProtect` encode in-tier access totals, but when the zero-count guard is bypassed the fallback scenario is a well-provisioned Zipfian tier where recency (LRU position) and frequency are already correlated: the LRU-front item is old and cold. Using `min_by_key(lfu_counter)` as a tiebreaker introduces noise by confusing low-counter-because-new with low-counter-because-cold, and the net effect is a slight regression rather than the expected improvement.
