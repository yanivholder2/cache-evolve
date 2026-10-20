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
