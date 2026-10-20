# Variant v003 Insight

## Helped

- hit-rate: Maintained exact parity with baseline (0.6652) by using a larger bounded eviction window (capacity/3 vs baseline's implicit unbounded search), confirming that window-size calibration alone can preserve hit-rate performance without frequency tracking.

## Hurt

(None)

## Takeaway

Variant v003 demonstrates that pure LRU with a fixed bounded-window eviction strategy (scanning the first min(capacity/3, 512) candidates from the LRU end) achieves identical hit-rate performance to the baseline. This validates the hypothesis that frequency tracking is not a strict requirement on this workload—window-size calibration provides sufficient selectivity through recency ordering alone. The variant's performance parity with identical metadata overhead (48 bytes per node) suggests that simpler LRU implementations may be preferable when frequency tracking adds complexity without measurable benefit.
