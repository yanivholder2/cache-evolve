# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Switching the hot tier (counter ≥ 3) from magnitude-2 to magnitude-1 decay — leaving counter-3 items at 2 instead of 1 and counter-4 items at 3 instead of 2 after a warm-lock event — produced no change in hit-rate on workload-a. This indicates that hot items landing at counter 1–2 after the parent's uniform magnitude-2 decay are still effectively protected by the cold-first scan on subsequent eviction calls, meaning the extra frequency signal preserved for the hot tier never changes which item gets chosen for eviction. The frequency gradient above the lukewarm tier is not a binding constraint under this workload and bucket geometry.
