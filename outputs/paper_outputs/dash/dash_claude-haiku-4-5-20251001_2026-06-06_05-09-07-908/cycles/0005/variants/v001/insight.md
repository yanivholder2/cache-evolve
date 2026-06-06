# Variant v001 Insight

## Hurt

- **hit-rate**: Lowering the promotion threshold to 1 causes aggressive over-promotion into the limited probing slots (only 8 buckets in the 75/25 geometry), diluting the hot-set quality as items without sufficient frequency evidence pollute the tier and subsequently get evicted back to target, reducing selection accuracy.

- **per-op-time (get)**: The threshold=1 gate promotes far more items into the probing bucket, creating dense collision chains that extend lookup paths significantly; the p99 latency shows the most severe regression (206 ns → 373 ns), indicating the probing tier becomes a bottleneck under increased occupancy, making worst-case lookups substantially slower.

## Takeaway

Lowering the frequency gate to threshold=1 crosses into counterproductive territory: it trades hit-rate and latency for the illusion of faster promotion. The 75/25 geometry's 8-bucket probing slot cannot absorb the flood of items that now qualify, creating severe contention that undermines the entire promotion design. Threshold=2 was already fairly selective; threshold=1 removes enough discrimination that false-positives dominate, showing the frontier where frequency gating becomes self-defeating on concentrated bucket geometries.

