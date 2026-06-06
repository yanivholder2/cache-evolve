# Variant v001 Insight

## Helped

- **hit-rate**: Aggressive 250k-operation decay allows the sketch to forget stale frequency patterns and respond more quickly to workload phase changes, yielding marginally improved hit-rate precision for eviction decisions.

## Hurt

- **per-op-time**: Calling `decay_if_needed()` on every `increment()` and `estimate()` operation adds a conditional branch and occasional O(depth×width) decay work, increasing latency by ~3–7 ns per operation.

## Takeaway

Adding sketch decay successfully improves hit-rate responsiveness, but the per-operation overhead of checking and periodically executing the decay mechanism negates the latency advantage the depth-3 baseline was designed to preserve. The hypothesis traded away latency gains for marginally better hit-rate precision—a trade-off that may not align with the goal of "preserving per-operation latency advantages."
