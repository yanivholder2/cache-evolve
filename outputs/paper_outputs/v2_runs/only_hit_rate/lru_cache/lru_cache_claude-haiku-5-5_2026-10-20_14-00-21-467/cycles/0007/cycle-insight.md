# Cycle 0007 Insight

## Cohort outcome

The cohort tried a ghost-set admission filter (v001), tail insertion of new keys (v002), and a frequency-aware eviction that picks the least-hit entry within a bounded four-entry tail window (v003). The Winner, v003, is the only variant that reports a hit-rate movement, and that movement is marginal.

## Patterns

- Only the frequency-aware tail-window eviction (v003) moved hit-rate, and it moved it slightly upward on workload-a at size 2048. Its gain is the one positive signal in the cohort, and its Winner status is consistent with that.
- The ghost-set admission filter (v001) left hit-rate unchanged. Its insight notes the fitness vector has no admission or rejection counts, so it cannot distinguish a filter that never engaged from one that engaged without effect.
- Tail insertion (v002) gave a hit-rate exactly equal to baseline. That is either a true null or a sign the insert-position change never affected evictions. This cycle's result conflicts with cycle 0005 (gain) and cycle 0006 (loss), so the tail-insertion evidence is not yet usable.
- v003 adds a per-entry hit counter whose memory cost is not in the fitness vector. Per-op time and metadata size were again not recorded, so the Winner's cost side is unknown. This is the same unmeasured gap carried since cycle 0001.

## Open questions

- Does the v003 gain survive on a second trace or a different cache size, or is it specific to workload-a at size 2048?
- Did the v002 tail-insertion overlay actually engage? Until a counter shows it did, its null cannot be read as evidence against the idea.
- Did the v001 admission counter ever reach the filter? If not, the ghost-set idea has not been tested at all.
- What do the per-entry counter and the tail-window scan cost in per-op time and metadata?

## Continuation

Pursue the bounded frequency-aware eviction (v003) as the lead lineage, but only after recording per-op time and counter memory and re-running it on a second trace or size. Verify engagement counters for the tail-insertion overlay (v002) and the ghost filter (v001) before ruling either idea in or out. Drop the ghost-set admission rule as written unless its counter shows it engaged.
