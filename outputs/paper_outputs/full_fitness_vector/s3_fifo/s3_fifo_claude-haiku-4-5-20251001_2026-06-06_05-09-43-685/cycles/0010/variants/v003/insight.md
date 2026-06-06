# Variant v003 Insight

## Helped

- **hit-rate**: Frequency-threshold gating on ghost-hit re-admission prevents low-frequency items (freq < threshold) from re-flooding the main queue, preserving space for small-queue items to graduate and improving overall hit-rate by 1.14%.

## Hurt

- **per-op-time (get)**: The extended ghost tuple structure `(K, u8)` and more complex ghost-hit detection logic (enumerate + tuple destructuring + frequency lookup) add overhead to the hot path even on misses, increasing get p50 by 36%.
- **per-op-time (put)**: The put path now performs additional frequency threshold comparisons during ghost-hit detection, adding latency overhead that manifests as a 6.8% slowdown in put p50.

## Takeaway

This variant successfully addresses v002's hit-rate regression by implementing selective re-admission: only items with demonstrated re-access patterns (freq ≥ threshold) are restored to the main queue, preventing crowding of the working set. The trade-off is clear: the more selective gating adds computational overhead to every put operation and complicates ghost lookup, resulting in a 36% increase to get latency. The win in hit-rate (+1.14%) comes at significant latency cost, suggesting that future refinements should either optimize the frequency tracking mechanism (e.g., via inline frequency storage rather than tuple lookup) or limit frequency checks to truly rare ghost-hit cases.

