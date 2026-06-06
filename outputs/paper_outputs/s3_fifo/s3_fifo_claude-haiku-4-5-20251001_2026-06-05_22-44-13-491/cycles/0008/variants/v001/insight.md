# Variant v001 Insight

## Helped

- hit-rate: Tightening the admission gate from 1/12 to 1/13 filters more aggressively at the entry boundary, marginally reducing the population of low-reuse items that transit the small queue and slightly increasing hit concentration in the main queue.

## Hurt

*(none)*

## Takeaway

The 1/13 ratio compounds with MAX_FREQ=5 to yield a marginal +0.0002 gain over 1/12, confirming that further structural selectivity does contribute—but at a diminishing rate. The hypothesis predicted that tighter admission could unlock larger compounding returns when paired with frequency protection, yet the actual gain falls below cycle 0007 v001's isolated 1/13 improvement of +0.0003 over 1/12. This suggests that the admission gate has reached a saturation point for this workload, where each increment in selectivity extracts progressively smaller improvements.
