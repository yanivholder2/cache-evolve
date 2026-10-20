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
