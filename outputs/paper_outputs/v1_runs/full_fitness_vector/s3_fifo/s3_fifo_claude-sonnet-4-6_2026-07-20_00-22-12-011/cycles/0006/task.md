# Task: cycle-insight-writer

Cycle: 0006
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0006

## Winners

- v001
- v003
- v002

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0006\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0006\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0006\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0006\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0006\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0006\hypotheses\H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle probed three independent axes: further small-queue shrinkage (v002), O(1) victim selection in `evict_main()` (v001), and a lower ghost-promotion threshold (v003). The cleanest win was v002, which continued the monotone small-queue reduction trend with no downside; v001 delivered a large latency gain at the cost of hit-rate; and v003 backfired, confirming the ghost channel is a necessary quality gate.

## Patterns

- **Small-queue shrinkage has not hit its floor**: Halving the fraction again (1/10 → 1/20) improved both hit-rate and get latency without any observed penalty — the monotone trend begun in cycle 0001 is still running clean on Zipfian workloads.
- **Deque churn, not scan length, was the latency driver in `evict_main()`**: Two-random-choices eliminated the `pop_back`/`push_front` memory-layout churn and halved get p99, but the associated loss of frequency-drain protection allowed hot entries to be discarded on a single bad draw — the hit-rate cost was material, not noise. The complexity class (O(n) vs O(1)) and the eviction *policy* were coupled; this variant untangled them in a way that confirmed each has independent value.
- **The ghost channel is a dual gate — frequency *and* demand confirmation**: Promoting at `freq > 0` short-circuited the ghost window's role as a re-request filter, flooding main with once-warm entries that lacked strong recency evidence. The paper's `freq > 1` threshold is not conservative bookkeeping; it enforces that main receives only keys that have been re-requested *after* ghost-listing, not merely re-accessed while still in small.
- **All three variants landed in the Winner list despite mixed fitness signals**: v001 hurt hit-rate and v003 showed no benefit alongside a hit-rate regression, indicating the evaluation fitness function may weight latency or aggregate score in ways that allow hit-rate regressions to be offset. This should be treated as a signal about fitness-function sensitivity, not as endorsement of those changes in isolation.

## Open questions

- Can a bounded second-chance scan — e.g., a fixed cap of 8–16 candidates rather than full O(n) traversal — recover most of v001's latency gain while preserving the frequency-drain protection that kept the hit-rate intact?
- Is there a floor to the small-queue shrinkage benefit, and does the ghost deque's expanded capacity (which tracks `main_size`) begin to inflate tombstone backlog pressure as `main_size` grows toward the full capacity?
- The winner designation for v003 (no benefits, one hurt) is anomalous — does the fitness function include a latency dimension that v003 happened to pass, or is this a threshold artifact worth investigating?

## Continuation

v002 (1/20 small-queue fraction) is the clear carry-forward winner and should become the new baseline; it is a clean, risk-free improvement on every measured axis. The next cycle's primary hypothesis should be a bounded second-chance scan in `evict_main()` — testing whether a shallow cap (rather than full O(n) or pure O(1) random) recovers hit-rate quality while still eliminating most of the deque-churn latency that v001 identified. Do not revisit the `freq > 0` promotion threshold; v003 conclusively demonstrated that the ghost channel's demand-confirmation function is load-bearing and the paper's `freq > 1` rule should remain fixed.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle tested three hypotheses: a bounded second-chance scan cap in `evict_main()` (v001), further small-queue shrinkage from 1/20 to 1/40 (v002), and a step-2 frequency decrement to accelerate victim selection (v003). None of the three improved on the prior baseline without a corresponding regression, and all three variants independently surfaced the same underlying bottleneck: a linear `position()` scan in `get()` and `put()` that was not targeted by any of these changes.

## Patterns

- **The bounded-scan hypothesis was falsified**: The cycle 0003 continuation proposed capping the second-chance loop to recover hit-rate quality from v001 of that cycle. Testing revealed the opposite: on Zipfian workloads the unbounded scan already terminates in the first few iterations because cold entries naturally accumulate at the tail of main, so a fixed cap delivers no churn reduction while introducing forced eviction of still-hot entries beyond the budget.
- **The `position()` scan is the dominant latency cost**: Both v001 and v002 independently named a linear key-lookup scan in `get()` and `put()` — not the second-chance eviction loop — as the dominant per-operation overhead. This cost grows with main-queue depth, which means every slot shifted from small to main (v002) worsens it, and every approach that leaves main large leaves this hotspot untouched.
- **Small-queue shrinkage has entered a diminishing-returns regime**: The monotone hit-rate improvement from halving the small-queue fraction continued at 1/40, but for the first time the latency penalty from a longer main queue was material rather than negligible — the two effects are approaching a crossover as main approaches full capacity.
- **Frequency acceleration amplifies eviction work rather than reducing it**: Draining frequency faster (step=2) caused moderately-warm entries to exit main prematurely, increasing the rate of ghost-hit-driven `evict_main()` calls. Total eviction cost rose even as individual scan lengths shortened — scan-length reduction and total-eviction-work reduction are decoupled and can oppose each other.

## Open questions

- Can the `position()` linear scan in `get()` and `put()` be replaced with an O(1) lookup via an intrusive node structure, a side-map, or a hashmap into the deque — and would eliminating it resolve the latency penalty that now shadows both the small-queue sizing lever and the eviction-loop improvements?
- Is there a minimum small-queue fraction below which the main-queue `position()` cost permanently dominates the hit-rate gain, and is 1/20 already near that threshold?

## Continuation

The `position()` linear scan in `get()` and `put()` has now been identified by two independent variants as the primary latency bottleneck; it should be the sole focus of the next cycle's hypotheses. The small-queue fraction should be held at 1/20 — the clean, unambiguous win from cycle 0003 — rather than pushed to 1/40 where scan overhead begins to cancel the hit-rate gain. Do not revisit the bounded second-chance cap (falsified) or frequency step acceleration (falsified); both hypotheses have been conclusively ruled out on Zipfian workloads.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants targeted the `position()` O(n) bottleneck named in cycle 0004, taking three different structural routes — dual side-maps, a miss-path membership guard, and a unified value+frequency map — and all three won; v003's full consolidation into a single `resident_map` was the only variant to improve on all three fitness axes simultaneously.

## Patterns

- **The `position()` diagnosis was fully confirmed across independent approaches**: Every variant that eliminated O(n) deque scans in `get()` and `put()` delivered large latency gains, validating cycle 0004's identification of that scan as the binding bottleneck. The fix is structurally robust — the gain appeared regardless of whether the approach used two separate HashMaps (v001), a guard HashSet (v002), or a single consolidated map (v003).
- **Miss-path scanning was the dominant fraction of total scan cost**: v002's guard approach — which preserved the position()-based hit path while only short-circuiting misses — still yielded a major latency win. This quantifies that cache misses (roughly 27% of accesses on this Zipfian workload), each previously paying two full O(n) sweeps, were a larger cost share than the hit-path scan itself.
- **Structural consolidation multiplied gains**: v003's single `resident_map<K,(V,u8)>` achieved O(1) value lookup, frequency update, hit detection, and miss detection in one hash probe while simultaneously shedding two separate `VecDeque<u8>` freq structures and the Entry value field. This compounding made it the only variant to win on latency, metadata-size, and hit-rate — the marginal hit-rate gain came from also removing the falsified SCAN_CAP=16 from its parent.
- **Small-queue shrinkage may have headroom again**: The cycle 0004 crossover (where 1/40 stopped being free) was attributed specifically to a longer main queue amplifying `position()` overhead. With that scan eliminated, the cost model for deeper main queues has changed and the prior crossover point may no longer apply.

## Open questions

- Now that `position()` is gone, what is the new dominant per-operation cost? Eviction-loop deque churn, ghost-channel overhead, and HashMap hash-and-probe cost on large working sets are all candidates, but their relative weight is uncharacterized.
- Is the small-queue fraction crossover from cycle 0004 an artifact of the scan overhead, meaning 1/40 (or smaller) is now viable again without the latency penalty that blocked it before?
- v001 uses two separate hash probes per `get()` (one into `freq_map`, one into `data`) while v003 uses one; at large working sets this round-trip difference may become measurable — has that cost been eliminated or merely shifted?

## Continuation

v003's `resident_map<K,(V,u8)>` should be the new baseline: it is the most complete structural elimination of the bottleneck and the only variant to win on all three axes. The next cycle's primary hypotheses should be (1) re-testing whether the small-queue fraction can be pushed below 1/20 now that main-queue depth no longer incurs position() overhead — the prior crossover was scan-driven and may not recur — and (2) profiling what the new dominant per-operation cost is with the scan tax removed, to identify the next target rather than assuming any of the prior axes remain relevant.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
