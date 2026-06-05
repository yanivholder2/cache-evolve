# Variant v001 Insight

## Helped

(none)

## Hurt

- **hit-rate**: Occupancy-adaptive destination selection adds conditional branching that does not improve performance; the deterministic skip-to-probing strategy already handles both congested and uncongested stash scenarios optimally.

## Takeaway

Adding occupancy-awareness (75% stash threshold to choose between probing and target insertion) provides no measurable improvement over the simpler deterministic approach. This suggests that the performance gain comes from consistent MRU positioning and tier acceleration, not from adapting the destination tier based on load. The parent variant's fixed strategy of always promoting stash hits directly to probing with back insertion is robust across cache states.
