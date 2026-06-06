# Variant v001 Insight

## Helped

- hit-rate: Dropping the 4th min-row while holding `SKETCH_WIDTH_FACTOR` at 16 left hit-rate essentially flat, confirming the premise that on a factor-16 sketch collision over-count is already low enough that the removed row contributed negligible admission accuracy.
- metadata-size: Cutting `SKETCH_DEPTH` from 4 to 3 removed a full row of counters, shrinking the `CountMinSketch` heap allocation by a quarter (one of four rows), even though the reported summary metric (the `Node` layout) is unchanged.

## Hurt

- per-op-time: Contrary to the hypothesis, `get` p50/p99 rose rather than fell — the one counter read+write saved per `increment`/`estimate` is a negligible slice of per-op cost, so run-to-run variance dominated and the measurement moved the wrong way.

## Takeaway

Removing one min-row from the factor-16 sketch buys footprint, not speed: hit-rate holds flat and the sketch heap drops a quarter, but the predicted hot-path/eviction-path speedup never materialized and per-op-time regressed. At this width the 4th row's per-access cost sits below the benchmark noise floor, so `SKETCH_DEPTH` is a sketch-size lever, not a per-op-time lever.
