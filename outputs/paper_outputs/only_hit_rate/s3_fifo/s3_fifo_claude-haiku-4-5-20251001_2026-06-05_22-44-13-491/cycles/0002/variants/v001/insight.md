# Variant v001 Insight

## Helped

- hit-rate: Extending MAX_FREQ from 4 to 5 allows items one additional eviction round in the main queue, continuing the second-chance mechanism's ability to retain moderately-reused items.

## Hurt

(none)

## Takeaway

Increasing frequency depth from 4 to 5 yields a measurable but marginal improvement in hit-rate. This demonstrates clear diminishing returns: while the prior 3→4 step showed substantial gains, the 4→5 extension captures much less additional value. The frequency-based second-chance mechanism continues working, but five levels appears near the point where further depth expansion offers diminishing benefit for this workload's reuse patterns.
