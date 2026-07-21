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
