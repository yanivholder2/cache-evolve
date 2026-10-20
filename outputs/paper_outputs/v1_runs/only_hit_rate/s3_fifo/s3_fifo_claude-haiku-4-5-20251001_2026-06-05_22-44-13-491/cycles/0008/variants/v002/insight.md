# Variant v002 Insight

## Helped

- hit-rate: Extending MAX_FREQ from 3 to 6 allows frequently accessed items in the main queue to survive more eviction rounds by sustaining additional frequency decrements, giving proven reuse patterns stronger second-chance protection before final eviction.

## Hurt

*(none)*

## Takeaway

The frequency-depth dimension retains measurable gains at extreme structural selectivity: pushing MAX_FREQ from 3 to 6 improves hit-rate by 0.0014 despite the extremely tight 1/12 admission gate, confirming that deeper frequency protection complements aggressive admission filtering rather than reaching saturation at this boundary.
