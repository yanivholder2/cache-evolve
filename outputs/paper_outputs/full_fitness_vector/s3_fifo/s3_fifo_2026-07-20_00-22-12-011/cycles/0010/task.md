# Task: cycle-insight-writer

Cycle: 0010
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0010

## Winners

- v003
- v001
- v002

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0010\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0010\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0010\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0010\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0010\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0010\hypotheses\H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
