# Variant v003 Insight

## Helped

- hit-rate: Doubling both sketch width (to 8× capacity) and depth (to 8 rows) reduces hash collisions in frequency estimation, enabling more precise distinction between hot and cold items during eviction.

## Hurt

(none)

## Takeaway

Combining width and depth improvements yields measurable hit-rate gains on Zipfian workloads. The compound effect suggests that frequency estimate precision benefits from both expanded bin coverage and stronger hash independence; neither dimension alone saturates the improvement available from the other.
