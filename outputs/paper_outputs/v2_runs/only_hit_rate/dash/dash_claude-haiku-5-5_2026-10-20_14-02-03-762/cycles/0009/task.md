# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0009

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried three reuse-tracking and admission tweaks: restricting reuse-counter increments to hits on entries last touched before the immediately preceding access (v001), halving rather than zeroing the reuse counter on normal-tier demotion (v002), and dropping probing-tier exits that have zero probing hits instead of moving them to the stash (v003). Only v002 is listed as a Winner, yet its own insight reports a change within noise on the one measured trace, so the Winner status is not supported by this cohort's evidence.

## Patterns

- All three changes are flat on the single measured trace. None of the reuse-counter or admission refinements moved hit-rate beyond noise on LIRS workload-a at size 2048.
- The reuse-counter lineage has now shown no signal in two cycles. The partial-credit-on-demotion idea (v002) and the back-to-back filter (v001) both returned parent-level results, which weakens the case for the saturating-counter direction from Cycle 0006 on this workload.
- The probing-tier admission filter (v003) also left hit-rate flat, so stash-pressure relief is not visible as a hit-rate lever here.
- No insight reports metadata-size or per-op-time, so the cost side of every variant remains unmeasured.
- In all three cases the overlay sources could not be read, so the insights describe intended changes rather than verified implementations.

## Open questions

- Why is v002 a Winner when its measured change is within noise? Is it winning against a different baseline, or carried forward?
- Does the reuse-counter direction produce any gain on a second workload or capacity, given three cycles of flat or marginal results on the one trace?
- Does the probing-tier filter's bypassed-entry count show any stash-pressure effect that hit-rate alone does not capture?
- Are the reset-on-demotion and back-to-back gating sites implemented as hypothesized?

## Continuation

Next cycle should stop refining reuse-counter increments, reset timing, and probing-tier admission on the single LIRS trace, since none has moved hit-rate. If the reuse-counter lineage is kept at all, test its best existing form on a second workload and capacity while recording metadata-size and per-op-time, and drop the back-to-back filter and the probing-tier admission filter.


### Cycle 0008

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
