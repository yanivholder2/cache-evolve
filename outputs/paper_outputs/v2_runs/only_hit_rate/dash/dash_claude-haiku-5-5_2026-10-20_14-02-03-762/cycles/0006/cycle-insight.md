# Cycle 0006 Insight

## Cohort outcome

The cohort tried three changes around the stash promotion gate and the victim-side bookkeeping: replacing the one-bit seen-in-stash marker with a 2-bit saturating reuse counter (v001), reordering the promotion gate so the marker is read only at high occupancy (v002), and replacing a slot scan in eviction with a per-bucket occupancy counter (v003). Only v001 is a Winner, and its hit-rate gain over the parent is marginal on a single trace.

## Patterns

- The saturating reuse counter is the only change with any hit-rate signal, and it is weak. Gating promotion on persistent reuse rather than a single re-hit may matter, but one workload and one run cannot separate it from noise.
- The reorder and the occupancy counter both matched their parent exactly. That is consistent with behaviour-preserving refactors, but neither variant recorded per-op-time or metadata-size, so their claimed speed and layout benefits are unmeasured.
- Across the cohort, no variant recorded metadata-size or per-op-time. The cost side of the reuse counter, which adds bits per entry, is therefore unknown.
- None of the three insights confirms the overlay diff. Each describes the intended change, so the implementations are not verified as written.

## Open questions

- Does the reuse-counter gain hold on workloads or capacities other than the LIRS trace at size 2048?
- Does the 2-bit counter's metadata cost justify any hit-rate gain it shows?
- Do the occupancy counter and the reordered gate reduce per-op-time once that metric is recorded?
- Is the counter reset on demotion and eviction implemented as specified?

## Continuation

Next cycle should pursue the saturating reuse counter from v001 as the single lead, and test it on more workloads and capacities while recording hit-rate, metadata-size, and per-op-time. Drop the gate reorder, which showed no effect. Keep the occupancy counter only as a cost-side check that records per-op-time, and verify the overlay diffs before drawing conclusions from any variant.
