# Cycle 0003 Insight

## Cohort outcome

The cohort tried two kinds of change: gating when entries move into the main table (a one-touch admission delay in v003, and a deferred stash-to-target promotion in v002), and reordering eviction ties by recency (v001). Only the promotion deferral (v002) shows a hit-rate gain, and it comes with a worse get tail. All three are labelled Winners, but none of them has a clean case on the dimensions the cohort measured.

## Patterns

- Keeping single-touch stash entries out of target buckets (v002) gave a small hit-rate gain, which fits the cycle 0001 concern about displacing hot target residents. Blocking first-touch admission entirely (v003) lost hit rate, so on this trace early admission matters.
- The two gating changes pull in opposite directions on hit rate. Delaying when a key enters the table protects capacity but costs reuse, while delaying promotion out of the stash does not. The cohort does not show where the boundary sits.
- Tie-breaking among equal eviction counters by recency (v001) moved no hit rate and added cost to the eviction path, so the order among tied slots does not decide evictions on this trace.
- Latency is again unreliable. v001 and v002 report per-op regressions, v003's put gain is not matched by its get-side movement, and v002's tail worsened even though its stated mechanism was a latency win. Winner status does not track these measurements.
- Every measured signal comes from one workload (workload-a). Both v002 and v003 say the trace type their hypotheses targeted (stash-dominant, scan-heavy) was not measured, so none of the cohort's directional claims generalise yet.

## Open questions

- Why are v001 and v002 Winners given their recorded per-op regressions and no measured hit-rate gain (v001)? The Winner basis is still not visible in the insights.
- Does v002's small hit-rate gain hold on a stash-dominant trace, and does its tail regression go away there?
- Does v003's hit-rate loss hold on scan-heavy traces, where the admission filter was meant to help?
- The v001 takeaway credits a counter-ranking effect to v003, but v003's insight describes an admission gate, not a counter. Is this a stale reference to the previous cycle's victim-selection direction, or a mislabelled Variant?

## Continuation

Next cycle should pursue the stash-to-target promotion deferral (v002) and re-measure it on a stash-dominant trace before building on its hit-rate gain. Pursue admission gating only if it is aimed at scan-heavy traces, and check it there rather than on workload-a. Drop recency tie-breaking (v001) and do not credit any latency gain from this cohort until it is re-measured.
