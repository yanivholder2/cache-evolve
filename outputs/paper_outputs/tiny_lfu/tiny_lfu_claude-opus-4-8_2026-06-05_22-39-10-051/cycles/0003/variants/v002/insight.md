# Variant v002 Insight

## Helped

- hit-rate: Rewriting `CountMinSketch::increment` to a conservative-update rule (`max(counter, min_estimate + 1)`, raising only cells at the minimum) trims overestimation of cold keys, nudging the `incoming_freq > victim_freq` gate a hair more accurate — but the gain is within noise, not a real lift.

## Hurt

- (none observed): CU increments stay monotonic and never withhold signal, so no admission decision regressed; no other dimension is measured.

## Takeaway

Stacking conservative update on the parent's already-wide sketch (width factor 16) bought essentially nothing: at that width, count-min cell collisions — the source of cold-key overestimation that CU corrects — are already rare, so the accuracy lever and the width lever are redundant rather than additive on this workload. CU's payoff would require a narrower, more contended sketch where overcounting actually distorts the admission gate.
