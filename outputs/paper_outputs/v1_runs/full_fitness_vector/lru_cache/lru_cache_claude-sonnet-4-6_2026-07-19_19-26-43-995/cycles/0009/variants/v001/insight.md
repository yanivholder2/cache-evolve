# Variant v001 Insight

## Helped

- **hit-rate**: Reducing `prob_cap` from `capacity/8` to `capacity/12` expands the protected queue's share of capacity from 87.5% to ~91.67%, allowing more frequently-accessed items to remain shielded from eviction and lifting the hit rate above the parent's value.

## Hurt

- **per-op-time**: The larger protected quota triggers demotion events more often when the protected queue overflows; the get p50 rose from the zero measurement-floor to a measurable 100 ns and put p99 doubled to 200 ns, showing that demotion pressure at this quota level is no longer sub-floor.

## Takeaway

Shrinking `prob_cap` to `capacity/12` does recover hit-rate relative to the `capacity/8` baseline, confirming that quota tightening continues to move the protected/probationary split in a direction that benefits Zipfian workloads. However, the hypothesis that per-op-time would stay at zero was falsified: at 91.67% protected capacity, overflow demotions are frequent enough to surface in latency. This places `capacity/12` in a similar tradeoff position as `capacity/16` — better hit-rate, but with a measurable latency cost — suggesting the zero-latency-cost ceiling is somewhere between 87.5% and 91.67% protected capacity.
