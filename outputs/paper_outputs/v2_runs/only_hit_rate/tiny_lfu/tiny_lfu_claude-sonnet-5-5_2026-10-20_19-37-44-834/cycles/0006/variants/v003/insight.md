# Variant v003 Insight

## Helped

- hit-rate: Unchanged relative to the parent, which is what H03 intended. Swapping links for `u32` slab indices left the lookup map, sketch, segment capacities and admission logic alone, so eviction and admission decisions did not change.

## Hurt

- metadata-size: The variant's fitness vector has no metadata-size entry, so the intended shrink of per-entry links from pointer width to `u32` was never measured. The change cannot be credited with a reduction.
- per-op-time: The variant's fitness vector has no per-op-time entry, so the claimed locality gain from a contiguous slab was never measured either.

## Takeaway

H03 is a behaviour-preserving refactor, and the evaluation confirms it preserved behaviour. Its goals, smaller metadata and faster ops, are unverified because only hit-rate was reported. The likely cause is that the metadata-size and per-op-time probes were not run, or did not work against the slab layout, so the result is incomplete rather than a failure. I could not inspect the overlay source, so this is inferred from the fitness vector alone. Re-run the probes before judging whether index-based links are worth keeping.
