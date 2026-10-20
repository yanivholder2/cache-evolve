# Variant v002 Insight

## Helped

## Hurt

- **hit-rate**: The decay loop resets `lfu_counter` toward zero after every eviction, which undermines the vacancy-gated frequency promotion added in v001 — items that had legitimately accumulated 2+ hits get their counters halved below the threshold, suppressing early probing promotions and leaving hot items stranded in the slower target tier on the stationary Zipfian workload.
- **per-op-time**: The post-eviction decay adds an O(bucket-size) scan that fires on every target-bucket overflow; on a small cache where evictions are frequent, this extra pass on top of the existing LFU-minimum scan in v001 doubles median operation latency.

## Takeaway

Adding counter decay to a stationary Zipfian workload removes the very frequency signal the policy relies on: items that are genuinely still popular have their counters periodically suppressed, which makes the vacancy-gated promotion path fire less often and lets the LFU victim-selection scan more easily evict items that simply haven't had time to re-accumulate hits since the last decay event. The decay also stacks a second O(n) scan at eviction time onto the already-costly v001 scan, lifting the p50 latency by the same margin the LFU scan introduced in the first place. Temporal decay may have value against non-stationary (shifting-popularity) traces, but it actively conflicts with v001's frequency-promotion mechanism on a stationary distribution.
