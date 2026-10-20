# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried three kinds of change: keeping the deferred stash-to-target promotion while cheapening its first-hit bookkeeping (v001), biasing victim selection by a saturating access counter (v002), and repacking the per-item record layout (v003). The promotion and victim changes both report hit-rate gains; the repack is hit-rate neutral and its per-op gain is unexplained. All three are Winners, and each rests on a single trace.

## Patterns

- Frequency-aware victim selection (v002) is the only change in the cohort with a clear hit-rate gain, and it matches the cycle 0002 and 0003 direction of protecting hot residents at eviction rather than at insert. Its cost lands on the put path, where full buckets now need a lowest-counter search.
- The deferred stash-to-target promotion from cycle 0003 survives here (v001) with its hit-rate gain intact. The change that matters is the cost of the first-hit state on stash hits, not the deferral itself: recovering that state moved the get tail back toward the parent's level.
- Record layout alone does not move eviction (v003): the repack left hit rate unchanged, which supports treating layout and replacement policy as separable.
- Per-op movements again outrun their mechanisms. v001 credits put-side tail movement its edit does not touch, and v003's get and put gains are larger than a size reduction obviously explains. This is the same pattern flagged in cycles 0001 to 0003.
- Every hit-rate signal in this cycle comes from workload-a. The stash-dominant and scan-heavy traces that earlier cycles asked for remain unmeasured.

## Open questions

- Does v002's counter-based victim gain hold on a second trace? Its insight refers to both workload-a and a LIRS workload, so it is unclear whether these are the same trace.
- Does v001's deferred promotion keep its hit-rate gain on a stash-dominant trace, which is where the cycle 0003 continuation asked it to be checked?
- Is v003's per-op gain caused by the packing, or by something else in the run? Re-measuring with only the layout change applied would settle it.
- What is the put-side cost of v002's victim search on full buckets, and does it grow with bucket size?

## Continuation

Next cycle should pursue frequency-aware victim selection (v002) and check it on a second, non-workload-a trace before building further on it, and it should look at cheaper ways to find the victim so the put-path cost does not grow. Keep the deferred promotion (v001) and test it on a stash-dominant trace as the cycle 0003 continuation asked. Drop further claims that record repacking (v003) moves hit rate, and do not credit any per-op gain from this cohort until it is re-measured.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried to narrow or cheapen the frequency-aware victim selection from cycle 0004: a two-choice sampled victim search (v001), the same counter restricted to stash-bucket victims on top of the deferred promotion (v002), and a record repack that was meant to shrink the segment (v003). None of the three reproduces cycle 0004's hit-rate gain, and the repack did not change the segment size it was meant to change. All three are labelled Winners, but each records a hit-rate or per-op regression.

## Patterns

- Narrowing the frequency-aware victim choice gave back its gain. Sampling two slots instead of scanning the bucket (v001) and restricting the counter to stash victims (v002) both left hit rate below the full-scan version from cycle 0004, so the retention effect depends on seeing the true lowest-counter resident.
- Cutting the victim search short did not make puts cheaper. v001's two-choice search raised put latency rather than bringing it toward the parent's level, which suggests scan length was not the main put-path cost. This partly answers cycle 0004's open question about put-side cost; the cost source is still unidentified.
- The frequency signal is not a cheap add-on to the deferred promotion. Stacking the counter on stash victims (v002) hurt both hit rate and both get and put tails, so the cycle 0004 combination with v001's promotion is not safe to extend without re-measurement.
- The repack did not land (v003). Segment and metadata size are unchanged, and hit rate is identical to the parent. Cycle 0004's per-op gain from the same layout idea therefore still has no mechanism behind it, and the size-reduction explanation offered there is not supported.
- Per-op movements remain unexplained. Get and put tails move in both directions across the cohort, and the attributions given are inferred rather than confirmed in source. This is the same pattern flagged in cycles 0001 to 0004.
- Every hit-rate signal again comes from workload-a. The second-trace checks requested in cycles 0003 and 0004 were not run here.

## Open questions

- Does full-scan lowest-counter eviction (cycle 0004 v002) keep its hit-rate gain on a second trace, and how much of its put cost is the scan itself?
- Is there a cheaper way to find the lowest-counter resident that keeps the full-scan ranking, such as maintaining the minimum incrementally?
- Why are all three variants Winners given their recorded regressions? The Winner basis is still not visible in the insights.
- Does the cycle 0004 per-op gain from the repack survive a layout change that actually reduces segment size?

## Continuation

Next cycle should go back to full-scan frequency-aware eviction from cycle 0004 (v002) rather than sampled or stash-only variants, and test it on a second, non-workload-a trace; focus on a cheaper way to find the victim. Keep the deferred promotion as the base. Drop the two-choice sampling (v001) and stash-only restriction (v002). Do not build on the repack (v003) until a variant verifiably shrinks the segment, and do not credit any per-op gain from this cohort until it is re-measured.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
