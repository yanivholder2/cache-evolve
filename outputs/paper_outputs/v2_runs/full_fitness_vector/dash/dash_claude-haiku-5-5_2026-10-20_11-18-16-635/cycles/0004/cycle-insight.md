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
