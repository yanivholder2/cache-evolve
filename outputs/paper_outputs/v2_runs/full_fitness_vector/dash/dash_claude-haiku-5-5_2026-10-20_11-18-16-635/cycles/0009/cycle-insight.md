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
