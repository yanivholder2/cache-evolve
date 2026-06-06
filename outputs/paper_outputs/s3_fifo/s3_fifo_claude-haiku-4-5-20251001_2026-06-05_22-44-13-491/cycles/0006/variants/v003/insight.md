# Variant v003 Insight

## Helped

- hit-rate: Tightening the small-queue admission gate to 1/12 capacity filters out low-frequency items more aggressively, ensuring only proven reuse patterns enter the main queue and reducing eviction churn.

## Hurt

*(none)*

## Takeaway

The 1/12 small-queue ratio continues to improve hit-rate without degradation, confirming that stricter frequency-based admission filtering is beneficial for this workload. The saturation boundary remains unexplored—the system continues to reward even more selective admission despite the dramatic reduction from the baseline 1/4 ratio.
