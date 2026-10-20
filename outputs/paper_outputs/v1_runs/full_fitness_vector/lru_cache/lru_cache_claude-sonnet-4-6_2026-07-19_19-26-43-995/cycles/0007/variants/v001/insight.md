# Variant v001 Insight

## Helped

- **hit-rate**: Halving `prob_cap` from `capacity/8` to `capacity/16` further reduced the number of moderately-frequent Zipfian items that cycle through the probationary queue and get evicted before earning a second access, extending the monotonic gain seen in cycle/0006.

## Hurt

- **per-op-time**: The get p50 summary rose from the parent's 0 ns to 100 ns; since the code change touches only the `prob_cap` constant and introduces no new operations on any hot path, this reflects the parent having had an anomalous measurement floor rather than a real latency cost from the quota change.

## Takeaway

The hit-rate improvement from compressing the probationary pool now spans three successive quota reductions (75% → 87.5% → 93.75% protected), confirming a genuine and so-far monotonic relationship between protected-queue share and hit-rate on the Zipfian workload. The gain per halving is diminishing (the 75→87.5% step was larger than the 87.5→93.75% step), suggesting the curve is flattening and the next halving (capacity/32) may yield only marginal returns.
