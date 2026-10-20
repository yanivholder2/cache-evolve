# Variant v003 Insight

## Helped

- **hit-rate**: Reducing small-queue capacity to 1/14 maintains stricter admission discipline, ensuring the main queue fills with proven frequent items rather than borderline candidates, improving cache effectiveness on this workload.

## Hurt

(no dimensions degraded)

## Takeaway

Dropping the small-queue admission ratio from 1/4 to 1/14 yields a measurable hit-rate gain without saturation penalties. The workload tolerates—and benefits from—this aggressive frequency-based filtering. The absence of degradation suggests that even at 1/14, entries promoted into the main queue are sufficiently high-value, and the cache remains far from an admission cliff. This reinforces the hypothesis that the saturation boundary lies beyond 1/14, justifying continued aggressive reduction in future cycles.
