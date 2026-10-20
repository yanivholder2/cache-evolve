# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0010

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tried three changes: collapsing the saturating reuse counter into the one-bit seen-in-stash marker (v001), adding an equivalence-preserving occupancy check that skips slot scans which cannot find a free slot (v002), and moving 128 slots from the stash back to the probing tier at fixed total capacity (v003). Only v003 is a Winner, on a small hit-rate gain over its parent on the single LIRS trace at size 2048.

## Patterns

- The capacity move is the only change with a hit-rate signal. Reversing the cycle 0008 shrink of the probing tier improved hit-rate, which is the opposite of what that shrink produced. Together the two results point to probing residency, not stash capacity, as the binding resource on this trace.
- The reuse-counter direction is again weak. Removing the counter's extra states (v001) produced a small drop just outside tolerance, so the counter may carry minor signal, but the effect is small and on one trace.
- The occupancy check (v002) matched its parent exactly, as a victim-preserving change should. Its cost benefit is unmeasured because per-op-time is absent from the fitness vector.
- No variant recorded metadata-size or per-op-time, so the cost side of the cohort is still unknown, including the bit cost of the reuse counter.
- None of the three insights confirms the overlay source. The v003 reading is tied to the stated change, not to verified constants, so the winning implementation is not yet checked.

## Open questions

- Is the v003 gain larger than trace-level noise, and does it hold on a second workload or capacity?
- Where is the probing-tier optimum? The gain from growing probing may flatten or reverse beyond 128 slots.
- Does the small v001 drop indicate a real loss of reuse-depth information, and is it worth the metadata it costs?
- Do the occupancy skip and the probing resize stay consistent with the stash-full retry and derived sizes?

## Continuation

Next cycle should pursue the probing-tier capacity increase from v003 as the lead, and test a few sizes around it on a second workload while recording hit-rate, metadata-size, and per-op-time. Drop the seen-in-stash collapse from v001, since it did not help and the reuse-counter lineage has shown no robust signal. Keep the occupancy check only as a cost-side check that records per-op-time, and verify the overlay diffs before treating v003 as settled.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
