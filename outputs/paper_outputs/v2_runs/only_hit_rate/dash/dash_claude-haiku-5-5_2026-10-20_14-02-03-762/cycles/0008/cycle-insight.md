# Cycle 0008 Insight

## Cohort outcome

The cohort tried three changes: gating probing-origin stash promotions on a saturated reuse counter (v001), skipping the reuse-counter store on hits to already-saturated entries (v002), and shifting 128 slots from the probing tier to the stash (v003). Only v002 is listed as a Winner, yet its own insight reports hit-rate unchanged from the parent and no per-op-time measurement, so the Winner status is not supported by this cohort's evidence.

## Patterns

- The reuse-counter direction is flat again. The saturated-counter promotion gate (v001) left hit-rate within noise on LIRS workload-a at size 2048, which is the fourth cycle in a row where refinements of reuse tracking show no hit-rate signal on that trace.
- The store-skip change (v002) matched its parent exactly. That fits a behaviour-preserving change, but its claimed cost benefit is unmeasured because per-op-time is not in the fitness vector.
- Shrinking the probing tier to grow the stash (v003) lost hit-rate. The probing tier does not look oversized, and stash pressure does not look like the binding constraint on this trace.
- No variant recorded metadata-size or per-op-time, so the cost side of the whole cohort is still unknown.
- None of the three insights confirms the overlay source. Each describes the intended change, so the implementations are not verified as written.

## Open questions

- Why is v002 a Winner when its measured hit-rate is identical to the parent? Is it carried forward for its cost claim, which has not been measured?
- Does any reuse-counter variant gain hit-rate on a second workload or capacity, given four cycles of flat or marginal results on one trace?
- Would a probing-tier capacity change in the opposite direction help, or is the split already at a reasonable point?
- Is the store-skip and saturated-gate logic implemented as hypothesized?

## Continuation

Next cycle should stop refining reuse-counter gates and store behaviour on the single LIRS trace, and should drop the probing-to-stash capacity shift. If any cost-side experiment continues, record per-op-time and metadata-size and verify the overlay diffs first. If the reuse-counter lineage is kept at all, test its best existing form on a second workload and capacity before proposing further variants.
