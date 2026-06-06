# Variant v003 Insight

## Helped

(none)

## Hurt

- hit-rate: The hybrid admission gate that combines frequency with a monotonically increasing time component creates a gate bias that grows unbounded, eventually admitting low-quality items as time progresses, which degrades discrimination between genuinely hot arrivals and noise.

## Takeaway

Extending the admission gate with a time-dependent component to mirror the eviction layer's frequency-recency hybrid did not translate to the admission layer. Unlike eviction, where age and frequency provide independent signals about item value, the proposed gate_score = frequency + (current_time / RECENCY_SCALE) lacks grounding: the time component grows monotonically regardless of the incoming item's actual recency, biasing the gate to admit more items over time rather than filtering based on access patterns. A successful admission hybrid would need to measure the incoming item's age (when it was last seen externally), not absolute elapsed time.
