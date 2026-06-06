# Variant v002 Insight

## Helped

_(none — hit-rate did not improve)_

## Hurt

_(none — hit-rate did not decline)_

## Takeaway

Adding a coarse `visit_count` tie-breaker to break IRR ties produced no change in hit rate on workload-a, meaning equal-IRR collisions either do not occur often enough in this LIRS-structured trace to affect which candidates are ultimately evicted, or when they do occur the positional order already happens to select the same victim. The IRR signal alone is sufficient to differentiate warmth on this workload, and layering a secondary frequency dimension on top of it neither recovers additional signal nor introduces noise.
