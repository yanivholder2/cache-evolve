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
