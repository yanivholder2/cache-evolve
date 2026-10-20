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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried three kinds of change: caching the eviction victim per bucket as an incremental minimum (v001), packing the `DashSegment` fields to shrink the segment (v002), and re-admitting recently evicted keys from a ghost ring (v003). Only the packed layout produced a measurable reduction, and it came with per-op regressions. The victim cache and the ghost ring both made the put path more expensive, and the ghost ring also lost hit rate. All three are labelled Winners, but none of them has a clean case on the dimensions the cohort measured.

## Patterns

- Maintaining the lowest-counter victim incrementally (v001) did not pay for itself. Hit rate and metadata size matched the full-scan parent, while put and get tails got worse. This answers the cycle 0005 question about a cheaper way to find the victim: keeping the minimum up to date on every counter write costs more than the scan it replaces, so the scan itself should stay.
- The layout change (v002) delivered the footprint reduction it targeted and left eviction order alone, which supports the cycle 0005 and 0004 view that layout and replacement are separable. But the packed fields added decode work on the get and promotion paths, and both operations slowed. A real size saving is not yet a usable win.
- Ghost-ring re-admission (v003) lost a small amount of hit rate on workload-a. The re-admitted keys appear to displace residents without adding re-referenced hits, which is the same displacement concern raised about promotion in cycle 0003. Its put-path bookkeeping on every insert and eviction is an added cost with no offsetting gain.
- Per-op movements again do not match their mechanisms. v003's reported get gain sits on a path the hypothesis did not touch, and the insight itself calls it unconfirmed. This is the same pattern flagged in cycles 0001 to 0005.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace and stash-dominant checks requested in cycles 0003 to 0005 were not run here.

## Open questions

- The Winner basis for all three variants is still not visible. v001 and v002 record per-op regressions, and v003 records a hit-rate loss.
- Can the packed `DashSegment` keep its footprint reduction without the get and promotion decode cost, for example by packing only fields that are not read on the hot path?
- Does the ghost ring's cost show up in metadata size once its bytes are counted? It is currently unmeasured, so the put-path cost is not fully accounted for.
- Does full-scan lowest-counter eviction (cycle 0004 v002) keep its hit-rate gain on a second trace? That check is still outstanding.

## Continuation

Next cycle should drop incremental victim caching (v001) and keep the full scan as the victim-selection base, and it should test the full-scan frequency eviction on a second, non-workload-a trace as the cycle 0005 continuation asked. Pursue the packed layout (v002) only if the next variant removes the decode work from the get and promotion paths while keeping the smaller segment. Drop ghost-ring re-admission (v003) unless a later variant shows a hit-rate gain and reports the ring's bytes in metadata size.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried three changes around the full-scan lowest-counter victim choice: an exact early exit from the victim scan on a zero counter (v001), halving every counter after each eviction (v002), and porting the saturating counter and full-scan victim choice onto the packed `Item<u64,u64>` record (v003). Only v003 showed a hit-rate gain, and it was partial. v001 and v002 have no helped dimensions, and all three are labelled Winners anyway.

## Patterns

- The full-scan, non-decaying counter remains the working base. Exiting the scan early (v001) left hit rate and metadata identical, so the victim choice was preserved. Decaying the counters (v002) lowered hit rate, which suggests the never-decaying counter is doing the retention work rather than just shielding stale residents.
- v003 is the first place the counter-based victim choice has been combined with a packed record. Hit rate rose, but by less than the hypothesis targeted, and metadata size did not change. Layout and replacement look separable again.
- Put cost still is not explained by scan length. The early exit did not make puts cheaper, which is the third cohort in a row where a shortened or cheapened victim search failed to lower put latency. The decay pass added put-path work and was the worst put result in the cohort, so the put-side cost appears to come from bookkeeping that runs on the eviction branch, not from the scan alone.
- Per-op movements again outrun their mechanisms. v001's get regression sits on a path the change does not touch, and v003's small get and put improvements are attributed to noise rather than the change.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace check first requested in cycle 0004 has now been deferred for four cycles.

## Open questions

- Does the v003 counter-plus-packed-layout hit-rate gain hold on a second, non-workload-a trace?
- Is the counter stored in spare bits or in the packed `u8` slot? The overlay source was not inspected, so this is unconfirmed, and it bears on the get and promotion decode cost raised in cycle 0006.
- Why are v001 and v002 Winners when they record no helped dimensions? The Winner basis is still not visible in the insights.

## Continuation

Next cycle should keep the full-scan, non-decaying counter as the victim-selection base and treat the packed-record port (v003) as the candidate to build on, but only after running the second-trace check that has been outstanding since cycle 0004. Drop counter decay (v002) and the early-exit scan (v001). Do not credit any per-op gain from this cohort until it is re-measured.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried three small, separate edits: doubling the saturating counter's per-hit increment (v001), packing the access counter into the `Item<u64,u64>` record (v002), and checking the target and probe buckets before the stash in `get` (v003). Only v002 and v003 moved a dimension, and each moved it in a different place: v002 reduced metadata size but hurt get and put, and v003 lowered get latency with no hit-rate change. v001 changed nothing measurable. All three are labelled Winners.

## Patterns

- Changing the counter's weight did nothing visible. Doubling the per-hit increment (v001) left hit rate and metadata identical to the parent, so the retention effect is not sensitive to the increment size at this setting. Its small get and put tail differences were attributed to noise.
- Packing the counter saved metadata but moved cost onto the hit path. v002 reproduced the footprint saving that cycle 0007 v003 suggested and kept hit rate flat, but the per-hit counter update on get and the packed decode on put both regressed. This is the same layout-versus-hot-path tension raised in cycle 0006, now with a concrete location: the counter update itself.
- Reordering the `get` lookup helped get latency without touching replacement. v003 removed the stash scan from the common target-hit path, which supports the cycle 0001 finding that the stash scan sits on the get path. Hit rate and metadata were unchanged. Its put movement is best read as noise, since the edit does not touch the put path.
- Stash-hit gets are still unmeasured. The stash-hit tail was the risk the hypothesis named, and it is not split out in the fitness vector, so the net effect of v003 is only partly known.
- Per-op attributions again go further than the evidence. v001 and v003 both explain small or unexplained movements as noise, and v002's hit-path explanation is inferred from the design, not confirmed against the overlay diff.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace check first requested in cycle 0004 has now been deferred for five cycles.

## Open questions

- Does the packed layout (v002) keep its metadata saving if the access counter stays unpacked for the hit increment and only cold fields are packed?
- Does the reordered `get` (v003) still win on stash-hit gets, once that tail is reported separately?
- Does a larger or smaller counter increment matter on a second trace, given that v001 was flat on workload-a?
- Why are all three variants Winners when v001 has no helped dimension and v002 and v003 each record regressions? The Winner basis is still not visible in the insights.

## Continuation

Next cycle should keep the full-scan, non-decaying counter as the base and pursue the `get` lookup reordering (v003) as the candidate most likely to carry forward, but only after the stash-hit tail is reported separately. Test packing with the hit counter kept unpacked (v002) before building on it. Drop the counter-increment change (v001) unless a second-trace run shows a gain. Run the second-trace check before crediting any per-op gain from this cohort.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tried three small edits around the full-scan, lowest-counter victim choice: sampling the counter increment on get hits (v002), reordering the `get` lookup to check target and probe buckets before the stash (v001), and seeding a new entry's counter at 1 instead of 0 (v003). None of the three moved a helped dimension. v002 lowered hit rate and did not improve get, and v001 and v003 left hit rate and metadata unchanged with small or unexplained per-op shifts. All three are labelled Winners.

## Patterns

- The victim choice is sensitive to how counts grow, not where they start. Seeding new counters at 1 (v003) was inert on hit rate and metadata, so the retention effect comes from hits accumulated after insertion. Sampling the increment on get hits (v002) did cost hit rate, which points the same way: hot keys need their counts to keep climbing.
- The premise that the per-hit counter write dominates get cost did not hold. v002 removed most of those writes and get latency rose instead. The get cost in the packed layout sits somewhere other than the counter update.
- The `get` reorder did not reproduce. Cycle 0008 v003 reported a get win on an earlier base, but v001 here moved get slower on both median and tail on the packed-counter parent. Hit rate and metadata were preserved, so the semantics held, but the latency claim is not carried forward.
- Put movements remain unexplained by their mechanisms. v001 and v002 both record put regressions on paths their edits do not describe, and v003's put shift is small enough to be noise. This is the same gap noted in cycles 0006 to 0008.
- Stash-hit gets are still not split out in the fitness vector, so the stash-hit tail that v001 flagged is still unmeasured.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace check first requested in cycle 0004 is still outstanding.

## Open questions

- What drives get latency in the packed layout, if not the per-hit counter write that v002 removed?
- Does reordering the `get` lookup help or hurt stash-hit gets once that tail is reported separately?
- Why do the put regressions appear in variants whose edits do not touch put, and is the overlay diff doing something the hypotheses do not describe?
- Why are all three variants Winners when none records a helped dimension? The Winner basis is still not visible in the insights.

## Continuation

Next cycle should keep the full-scan, non-decaying counter and focus on how counts grow after insertion, not on their starting value or on sampling them. Drop the insert-time seed change (v003) and the on-hit sampling (v002). Do not carry the `get` reorder (v001) forward until the stash-hit tail is reported separately and the get regression is re-measured. Run the second-trace check before crediting any per-op or hit-rate result from this cohort.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

The cohort tried three changes around the full-scan, lowest-counter victim choice: counting overwrites of resident keys as reuse (v001), a per-slot tag used as a probe-time filter (v002), and breaking counter ties by residency class instead of age (v003). None produced a clear helped dimension. Only v003 moved hit rate, and only marginally on workload-a. All three record slower get latency, and all three are labelled Winners anyway.

## Patterns

- Get latency regressed in all three variants. v002's get slowdown has a plausible path (the tag read and branch per probe), but v001 and v003 both touched paths their hypotheses did not describe as get-side. A shared cause, such as overlay placement or measurement variance, is more likely than three independent mechanisms.
- Put latency rose in all three. v001's rise is expected from its added increment. v002's and v003's are attributed to bookkeeping and an extra comparison on the eviction scan. Together with cycles 0006 to 0009, this reinforces that put cost is not explained by the victim-scan or per-write logic the hypotheses name.
- Hit rate is largely insensitive to the signals tried. Counting overwrites (v001) and the tag filter (v002, which lowered hit rate) did not help. The residency tie-break (v003) is at most a weak lever. Retention still appears to come from hits accumulated after insertion, consistent with cycle 0009.
- Keys are u64 in the packed record, so any change that saves a full key compare (v002) recovers little work. Per-slot filters only pay if key comparison is expensive here.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace check first requested in cycle 0004 is still outstanding.
- The overlay source was not inspected in any variant, so every mechanism above is inferred from the hypotheses, not confirmed against the diff.

## Open questions

- Is the get-path slowdown common to all three variants, and if so, is it an overlay placement or measurement effect rather than a property of any one change?
- Does the residency tie-break (v003) hold its marginal hit-rate gain on a second trace?
- Why did the per-slot tag (v002) lower hit rate when it was meant to leave eviction and promotion untouched? Was the tag mis-maintained on promotion, or did it turn real hits into misses?
- Why are all three variants Winners when none records a clearly helped dimension? The Winner basis is still not visible in the insights.

## Continuation

Next cycle should keep the full-scan, non-decaying counter as the victim-selection base and stop spending hypotheses on overwrite counting (v001), per-slot tag filters over u64 keys (v002), or residency-class tie-breaks (v003) unless the second-trace check shows a gain. Before any further variant is judged on get or put latency, re-measure the parent to determine whether the get slowdown is shared across this cohort. Do not credit any per-op or hit-rate result from this cohort until the second-trace check has been run.


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
