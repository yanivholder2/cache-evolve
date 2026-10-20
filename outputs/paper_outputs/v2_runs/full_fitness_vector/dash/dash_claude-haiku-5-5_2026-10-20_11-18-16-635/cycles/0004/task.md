# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0004

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0004/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried two kinds of change: where promoted entries land in their target bucket (insert position), and how the get path orders its tier lookups (stash scan versus target and probing scans). All three Variants were Winners, but they moved different dimensions, so the Winner label rests on different evidence in each case.

## Patterns

- Insert-position changes act on hit rate. Promoting at MRU protects the promoted key from the next eviction, but it can also displace a hot resident of a full target bucket, so the effect depends on how full the buckets are.
- The get-path reordering is the only change whose stated mechanism is latency. Skipping the stash scan on common target and probing hits is the cohort's clearest per-op improvement.
- Several Variants credit per-op-time movement that their own code paths do not explain, and put-side timing moved in more than one Variant. Treat latency gains from this cohort as unconfirmed until they are re-measured.
- The one Variant with a recorded hit-rate loss (v001) was still a Winner, so the Winner criteria are weighting latency over hit rate. Whether that is intended is unresolved.

## Open questions

- Does the MRU insert in the probing-recovery path lose hit rate on workloads with fuller target buckets, and is the v001 hit-rate loss stable across traces?
- How much of the get-latency gain from reordering the tier lookups is real, and what does stash-hit latency look like once it pays for the two earlier misses?
- Are the unexplained put and per-op movements measurement noise?

## Continuation

Next cycle should pursue the get-path reordering and check that its latency gain holds on stash hits and across reruns. Pursue insert-position work only with a guard against displacing hot target residents, since plain MRU promotion trades hit rate for latency. Drop further claims about put-side or per-op gains until they are re-measured.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort split into two families: insert-position gating on target-bucket fullness (v001), and changes to victim selection and stash handling (v002, v003). Only v003's change clearly targets hit rate, and its gain is measured on a single workload. All three are labelled Winners, but v001 and v002 each carry recorded regressions that their Winner status does not obviously offset.

## Patterns

- Gating promotion insert position on fullness moved the wrong way: placing promotions at the front of full buckets cost hit rate, so promotions into full buckets still need the protection the back-of-queue insert gave them.
- Protecting hot residents works better at eviction than at insert. Biasing the victim choice by a small saturating access counter is the only change that reports a hit-rate gain, which fits the cycle 0001 concern about displacing hot target residents.
- Stash-path bookkeeping does not pay on this workload. Adding an occupancy check to the get path and maintaining it on write paths raised get and put latency, which implies stash buckets are rarely empty here.
- Latency claims are again unreliable. v002's put regression is outside what its get-path change can explain, and v003's per-op gain is larger than a victim-only change would normally account for.

## Open questions

- Is v001's hit-rate loss stable across traces, and does the loss come only from full buckets?
- Why does v002 count as a Winner given its recorded get and put latency regressions? The Winner basis is not visible in the insight.
- Does v003's hit-rate gain hold on workloads other than workload-a?
- Is v003's per-op improvement real, or noise like the unexplained movements in cycle 0001?

## Continuation

Next cycle should pursue frequency-aware victim selection (v003's direction) and check it across more than one workload before building on it. Drop insert-position gating on fullness (v001) and stash occupancy fast paths (v002) unless new evidence shows empty stash buckets are common. Treat any per-op latency gain as unconfirmed until it is re-measured.


### Cycle 0003

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
