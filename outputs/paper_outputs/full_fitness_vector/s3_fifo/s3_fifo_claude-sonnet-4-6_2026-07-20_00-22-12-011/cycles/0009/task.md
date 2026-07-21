# Task: cycle-insight-writer

Cycle: 0009
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0009

## Winners

- v001
- v003
- v002

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0009\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0009\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0009\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0009\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0009\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0009\hypotheses\H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
