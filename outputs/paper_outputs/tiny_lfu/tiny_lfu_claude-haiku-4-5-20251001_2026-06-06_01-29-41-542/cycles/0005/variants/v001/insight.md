# Variant v001 Insight

## Helped

(none)

## Hurt

(none)

## Takeaway

Increasing the frequency offset constant from 1.0 to 2.0 in the victim-selection scoring formula produced no measurable change in hit-rate on workload-a. The larger denominator was intended to widen the score spread between low-frequency items and provide finer discrimination, but the eviction decisions remain behaviorally equivalent on this trace. This suggests the original offset of 1.0 was already sufficiently tuned for this workload's access patterns, or the change affects only edge cases that do not influence overall performance.
