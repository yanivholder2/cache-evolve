# Variant v002 Insight

## Helped

- hit-rate: Non-linear sqrt-based frequency damping in eviction scoring reduces the dominance of high-frequency items, allowing age-recency to more effectively discriminate among cold and moderately-accessed items.

## Hurt

(none)

## Takeaway

The sqrt-dampened frequency scoring achieves a modest hit-rate improvement by more evenly balancing frequency and age in victim selection. Instead of allowing very-hot items to dominate victim protection, the square root scaling lets recency participate equally in the decision, better identifying truly cold victims when frequency differences are small. This suggests that LFU variants may benefit from sub-linear rather than linear frequency scaling in mixed-temperature workloads.
