# Task: cycle-insight-writer

Cycle: 0007
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0007

## Winners

- v003
- v001
- v002

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0007\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0007\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0007\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0007\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0007\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0007\hypotheses\H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants explored sizing and layout levers that cycle 0005 left open: pushing the small-queue fraction below 1/40, widening the ghost queue beyond `main_size`, and decomposing the unified `resident_map` into parallel hashmaps. None found a positive outcome — every variant registered a regression on at least one primary dimension — making this a clean falsification cycle. All three variants nonetheless appear in the Winners list, continuing the anomaly first observed in cycle 0003.

## Patterns

- **The small-queue floor has been reached at 1/40**: Tightening to 1/80 produced a marginal hit-rate uptick but a significant latency regression from more frequent `evict_small()` calls — the overhead of faster small-queue turnover now outweighs the additional main-queue residency benefit. The crossover that cycle 0004 attributed to `position()` scan length has a separate instance at this fraction boundary, independent of the scan.
- **Ghost capacity beyond `main_size` is load-bearing only when the cold tail has recency**: Doubling the ghost window accumulated useless tombstones rather than recapturing evicted keys, hurting both hit-rate and put latency via the lazy-deletion drain loop. The Zipfian cold tail simply does not re-request within the extended window; the original `main_size` bound is effectively tight.
- **The unified `resident_map` layout is structurally optimal for this workload**: Splitting it into `data<K,V>` and `freq_map<K,u8>` to improve read-path cache-line density was offset by doubled write overhead on every admission and eviction. The cycle 0005 v003 consolidation is confirmed as the right data-structure choice, not a candidate for decomposition.
- **All three known optimization levers are now exhausted or at their floor**: scan elimination (cycle 0005), queue fraction (1/40 confirmed optimal here), ghost sizing (confirmed optimal at `main_size`), and map layout (consolidated `resident_map` confirmed optimal). No incremental improvement was found on any of them.

## Open questions

- What is the current dominant per-operation cost now that all the previously identified bottlenecks have been resolved? Hash-probe collision behavior, eviction deque memory layout, and ghost-set `HashSet` membership tests are candidates but have not been profiled.
- The Winners list again includes variants with net-negative fitness profiles — is the fitness function weighting a dimension (e.g., a specific latency percentile or aggregate score) that these variants happened to pass, and should that weighting be revisited?

## Continuation

The three axes that have driven all gains since cycle 0001 — small-queue fraction, ghost capacity, and per-operation data structure layout — are now at or past their practical optima on this Zipfian workload. The next cycle must pivot to identifying a genuinely new bottleneck rather than probing exhausted levers. The most promising approach is to instrument or reason about what the current dominant per-operation cost is: candidates include the `evict_main()` deque memory-access pattern under frequency churn, `ghost_set` HashSet probe overhead on every `put()`, and hash-collision behavior at high load factors. The baseline for all future cycles should remain cycle 0005/v003's consolidated `resident_map` with a 1/20 small-queue fraction; do not revisit 1/40 or smaller fractions, the `2 × main_size` ghost bound, or the split-map layout — all three have been conclusively ruled out.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
