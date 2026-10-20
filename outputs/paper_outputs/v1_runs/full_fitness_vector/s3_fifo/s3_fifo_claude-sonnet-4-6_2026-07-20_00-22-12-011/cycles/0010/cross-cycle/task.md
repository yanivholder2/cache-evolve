# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted distinct structural axes — queue-size ratio, ghost-lookup algorithmic complexity, and entry memory layout — and all three won over the baseline. The Winners share no single change but collectively demonstrate that the baseline S3-FIFO leaves headroom on multiple independent dimensions simultaneously.

## Patterns

- **Small queue was over-provisioned**: Reducing the small-queue fraction from 1/4 to 1/10 improved both hit rate and per-operation latency, indicating the original split over-invested in admission filtering at the expense of main-queue residency and scan length.
- **A dominant linear scan in `put()`**: The ghost-queue membership check was an O(n) scan that nearly halved median put latency when replaced with an O(1) hash-set oracle — the single highest-leverage algorithmic fix in the cohort.
- **Entry struct size is a meaningful lever**: Moving the frequency counter out of the entry struct achieved a 33% per-entry size reduction and delivered an unexpected latency benefit via improved cache-line utilization during the scan-heavy eviction path.
- **Fixed overhead amortizes cleanly**: Both v002 and v003 grew the `S3Fifo` control struct (HashSet header and parallel freq deque headers respectively), but that overhead is per-instance and constant, not per-entry, so it is negligible at any practical cache capacity.
- **The three improvements are orthogonal**: Each change touches a non-overlapping code path (sizing formula, ghost-check in `put()`, entry layout + eviction loops), making them safe candidates for combination.

## Open questions

- What is the optimal small-queue fraction, and does it depend on workload skew — or is there a threshold below which shrinking it further hurts cold-miss handling?
- v002 still calls `ghost.retain()` (O(n)) on a ghost hit to remove the key from the deque; the hot path for ghost hits is not yet O(1) end-to-end.
- Combining v001 + v002 + v003 may expose interactions (e.g., a smaller small queue shifts eviction pressure, changing how often the ghost set is queried); composite behavior should be validated.

## Continuation

All three strategies are worth carrying forward as a combined baseline for the next cycle — they are orthogonal and collectively represent a materially improved S3-FIFO. The next cycle should prioritize: (1) a composite variant merging all three wins, and (2) eliminating the remaining O(n) `ghost.retain()` call in v002's ghost-hit path, which is the last known linear-scan hotspot in `put()`.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle validated cycle 0001's open threads: the composite variant (v001) merged all three prior wins plus lazy ghost deletion and improved all three fitness dimensions simultaneously, while the two focused variants (v002, v003) confirmed each isolated change is independently additive. All three variants won.

## Patterns

- **The last O(n) hotspot is gone**: Replacing `ghost.retain()` with lazy tombstoning — removing the key only from the HashSet on ghost hit, deferring deque cleanup to the eviction capacity guard — cut put tail latency by 5× in the surgical variant and was the dominant gain in the composite as well. Amortizing per-hit cleanup across future evictions is a consistently powerful pattern here.
- **Orthogonality held at composition**: Queue sizing (1/10 fraction), parallel freq deques, and O(1) ghost operations all touched disjoint code paths in cycle 0001 and continued to stack cleanly when combined in v001 — no interaction penalty was observed.
- **Queue sizing introduces a put trade-off**: v003 confirmed that enlarging the main queue by shrinking small from 1/4 to 1/10 accelerates reads (shorter small-queue scan, more hot-set residency) but nudges put latency upward because the second-chance loop in `evict_main()` must traverse a longer queue to find a zero-frequency victim. This is the first observed cross-dimension tension from the sizing change.
- **Lazy deletion leaves deque slack**: Ghost tombstones may temporarily accumulate in the deque between capacity-guard sweeps. Under the current policy the deque can transiently exceed the live-entry bound; the guard drains them in future evictions, but the backlog is uncharacterized.

## Open questions

- Is the put latency increase from a larger main queue (observed in v003) masked in v001 by the O(1) ghost-hit path, or does it persist at higher ghost-hit rates?
- How large does the tombstone backlog in the ghost deque grow under workloads with dense ghost hits before the capacity guard catches up?
- The `evict_main()` second-chance scan is now the most expensive remaining loop — are there sub-linear victim selection strategies (e.g., approximate frequency indexing, segmented clock) worth probing?

## Continuation

v001 (composite: 1/10 small queue + parallel freq deques + O(1) ghost + lazy deletion) should be the new baseline for all further cycles; it dominates on every axis and eliminates all known O(n) paths. The next hypotheses should focus on the `evict_main()` second-chance scan, which is now the primary loop cost, and on characterizing whether the 1/10 small-queue fraction is at or near the optimal point for Zipfian workloads — particularly whether further shrinking it worsens cold-miss handling or puts pressure on the tombstone backlog.


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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle probed three largely independent axes: hash load-factor pre-sizing (v001), ghost-hit frequency initialisation (v002), and a wider second-chance window via a higher `MAX_FREQ` cap (v003). All three variants landed in the Winners list despite producing neutral or net-negative fitness profiles, continuing the anomaly from prior cycles. The cohort's primary value is falsification: each hypothesis closed off a candidate root cause rather than opening a new gain.

## Patterns

- **Hash-collision depth is not a bottleneck at any capacity**: Pre-sizing all maps to hold a ~50% load factor had zero measurable effect, definitively ruling out probe-chain length as the source of the latency regression associated with the 1/80 small-queue fraction. v001 simultaneously identified the true driver: `evict_small()` call-frequency, not per-call overhead. The cost model for tight small-queue fractions is invocation-rate dominated, not hash-efficiency dominated.
- **Ghost-hit frequency initialisation at `freq=1` is harmful on Zipfian**: Granting re-promoted entries one free second-chance pass extended the `evict_main()` scan depth and displaced hotter entries, lowering both hit-rate and put-path latency. The Zipfian cold tail, even when ghost-cached, does not reliably re-access soon enough to justify the deferral. `freq=0` on ghost admission is confirmed correct.
- **`MAX_FREQ=3` is already near the Zipfian hot-set protection ceiling**: Widening to 5 produced only a marginal hit-rate gain, indicating the hot set was already well-retained at the lower cap. The latency cost of proportionally deeper `evict_main()` loops outweighed the benefit, making this a negative tradeoff. Future hit-rate experiments on Zipfian should look beyond `MAX_FREQ` tuning.

## Open questions

- v001 pinpointed `evict_small()` invocation frequency as the binding cost for high-eviction configurations, but what is the most expensive operation *within* each `evict_small()` call at elevated rates — deque churn, ghost-set membership tests, or data-map removal?
- The Winners list has now included net-negative variants in multiple consecutive cycles; does the fitness function apply a scoring threshold or aggregate weighting that these variants happen to clear, and does that weighting reflect the intended optimisation target?

## Continuation

Cycle 0007 is a second consecutive falsification cycle: hash load factor, ghost-hit initialisation, and `MAX_FREQ` ceiling are all ruled out as improvement levers on Zipfian workloads. The next cycle should pivot away from these exhausted axes and target the `evict_small()` call-rate problem that v001 surfaced as the true latency driver. Promising directions include batching or amortising `evict_small()` work across multiple `put()` calls, or reducing the net number of eviction events by tuning admission selectivity rather than post-admission retention. The baseline remains cycle 0005/v003's consolidated `resident_map` with a 1/20 small-queue fraction; do not revisit 1/40 or smaller fractions, split-map layouts, higher `MAX_FREQ` values, or `freq=1` ghost initialisation.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle probed three directions surfaced by prior falsification rounds: reverting the `MAX_FREQ` ceiling (v001), widening the small-to-main promotion threshold (v002), and batching `evict_small()` work to reduce call-rate overhead (v003). All three variants landed in the Winners list; all three produced neutral or net-negative fitness profiles, making this a third consecutive falsification cycle.

## Patterns

- **Batch-drain eviction is super-linear in cost, not linear**: v003's hypothesis — that concentrating many evictions into one `put()` would leave the surrounding calls eviction-free and lower p50 — was falsified decisively. The burst variant made put p50 and p99 significantly worse. Eviction cost does not amortise across a batch; each incremental eviction in a sequence carries overhead that compounds rather than averages out. One-at-a-time eviction is confirmed as the correct policy for this implementation.
- **freq=1 entries do not benefit from main-queue protection on Zipfian**: v002's wider promotion gate admitted more freq=1 entries to main, increasing churn without recovering hits. These entries are too sparse or too short-lived in the Zipfian access pattern to justify the additional `evict_main()` scan depth they impose. The `freq > 1` gate is confirmed correct.
- **`MAX_FREQ` improvements are only visible in put tail latency, not the primary fitness signal**: v001's rollback to `MAX_FREQ=3` recovered put p99 measurably, but get p50 — the dimension that anchors the fitness summary — did not move. This dissociation confirms that the remaining latency headroom lies in the get path, not the eviction path. Any eviction-side tuning will continue to be invisible to the primary signal.

## Open questions

- With batching ruled out and call-rate identified as the `evict_small()` bottleneck, is there a structural approach — such as admission selectivity that reduces the net number of small-queue overflows rather than amortising their cost — that avoids both the invocation-rate penalty and the burst-cost penalty?
- The primary fitness signal is anchored to get p50, which has not moved in multiple cycles. What is the dominant per-operation cost on the get path specifically? All optimisation effort has targeted put and eviction paths.
- The Winners anomaly (net-negative variants winning) has persisted across at least five cycles — does the fitness threshold capture something other than the primary latency and hit-rate axes?

## Continuation

Three consecutive falsification cycles have exhausted the eviction-side improvement space: batching is ruled out, promotion threshold widening is ruled out, and `MAX_FREQ` tuning is confirmed near-exhausted. The next cycle must shift focus to the get path, which is where the primary fitness signal actually measures and where no targeted work has yet been done. Candidates include the `resident_map` lookup cost under high collision pressure, branch-predictor behavior in the hit-path conditional chain, and whether the ghost-set membership test on every `put()` is inflating get-interleaved latency. Do not revisit `evict_small()` batching, freq=1 promotion, `MAX_FREQ` tuning, split-map layouts, or fractions below 1/20.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle made its first targeted attempt on the get path, probing three distinct micro-optimisations: halving probe count via map consolidation, swapping to a lighter hasher, and eliminating redundant freq writes for saturated entries. All three are Winners; none moved get p50, and two produced net latency regressions — making this a fourth consecutive falsification cycle, now on the get-path axis.

## Patterns

- **The Zipfian hot set's L1 residency renders probe-count and hasher optimisations invisible**: Because the hot keys are accessed constantly, their map buckets stay in the highest cache level regardless of layout or hash algorithm. Halving probes (map merge) and substituting a simpler hash function therefore produce no measurable latency benefit — the cache miss being targeted simply does not occur on this workload.
- **Unconditional freq writes are load-bearing for eviction latency**: Skipping the freq write on saturated entries reduced the write-touch rate on those slots, allowing them to cool between `evict_main()` calls. When eviction subsequently scanned those freq entries, it paid a cache-miss penalty that was absent with the unconditional write. This is the inverse of the usual "fewer writes = faster" intuition: here, frequent re-dirtying keeps critical lines warm and is a net benefit.
- **SipHash is no slower than FNV-1a at this key size and cache scale**: The simpler hasher's theoretical advantage in CPU instruction count did not translate to wall-clock savings, suggesting SipHash's integer-optimised path is already at or near the throughput ceiling for the measured access rate.
- **Struct footprint shrank in two variants but fitness did not move**: Both the map-merge and the hasher-swap eliminated metadata bytes from the per-instance struct. These structural gains are real but orthogonal to the primary latency signal, confirming that per-instance size is not on the critical path at the tested cache capacity.

## Open questions

- With probe count, hash complexity, and write frequency all ruled out for get p50, is the get-path measurement itself the bottleneck — e.g., benchmark overhead, timer resolution, or workload construction — rather than any cache implementation detail?
- The Winners anomaly has now persisted across at least four consecutive cycles including this one; if the fitness threshold admits net-negative variants, does the current selection pressure meaningfully guide the search?

## Continuation

All three obvious get-path micro-optimisations — probe reduction, hasher replacement, write elimination — have been falsified in a single cohort. The L1 residency of the Zipfian hot set appears to make any approach targeting cache-miss reduction on the hot path structurally unreachable at this scale. The next cycle should either challenge the workload assumptions (e.g., probe the cold or medium-frequency tiers where cache misses do occur) or investigate whether the ghost-set membership test on every `put()` — the one per-operation cost that has not been directly targeted — is the remaining movable lever. Do not revisit map-merge vs. split layouts, custom hasher swaps, or write-skip guards for saturated entries; all three are exhausted on this workload.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

After four consecutive falsification cycles, this cohort produced two clean wins and one instructive tradeoff: v001 collapsed the ghost-membership probe into a unified state map (halving put latency), v003 raised `MAX_FREQ` to 4 (recovering hit-rate at zero latency cost), and v002 demonstrated that admission-sketch filtering cuts `evict_small()` invocations but introduces a hit-rate regression from warm-cold keys.

## Patterns

- **The two-probe put pattern was the dominant put-path cost, not eviction work itself**: Replacing the `data.contains_key` + `ghost_set.contains` pair with a single unified `state_map.get()` halved put p50 and p99. The ghost-set membership test on every `put()` — called out as an unexamined lever in cycle 0009 — was indeed the remaining movable cost, and eliminating the separate ghost structure resolved it cleanly. The struct-size reduction is a secondary bonus.
- **`MAX_FREQ=4` is a free hit-rate improvement over `MAX_FREQ=3`**: The extra frequency credit reduces premature eviction of medium-frequency entries without any detectable scan overhead. The `MAX_FREQ=5` regression from cycle 0007 was a threshold effect, not a smooth curve; 4 sits in a plateau where the benefit is real and the cost is below measurement resolution.
- **Admission selectivity is effective but workload-a is not a pure one-hit-wonder distribution**: The sketch filter confirmed that `evict_small()` call-rate is a genuine lever on put latency, but strict first-seen rejection over-filters — workload-a contains enough warm-cold traffic to produce measurable hit-rate loss. The direction is valid; the threshold is too aggressive.

## Open questions

- With put latency now halved by the unified map, are put p50 and get p50 now in the same order of magnitude? If get p50 remains the primary fitness signal and is still not moving, is it now at or near the implementation floor for this workload and algorithm?
- Can the admission-sketch threshold be softened (e.g., require two visits before skipping eviction, or use a shorter reset interval) to recover the hit-rate regression while retaining most of the `evict_small()` call-rate reduction?

## Continuation

Cycle 0010 breaks the falsification streak with two structural wins: the unified `state_map` (v001) and `MAX_FREQ=4` (v003) should both be carried forward as the new baseline. The next cycle should focus on refining the admission-sketch approach from v002 — specifically softening the rejection threshold from first-seen to two-visit, or decaying the sketch more aggressively, to close the hit-rate gap while keeping the eviction-rate reduction. A secondary thread worth opening is whether any remaining get-path cost exists now that put-path costs have been substantially reduced, or whether get p50 has genuinely reached the measurement floor. Do not revisit the split-map layout, `MAX_FREQ` values above 4, ghost `freq=1` initialisation, batch-drain eviction, or hasher swaps.


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
