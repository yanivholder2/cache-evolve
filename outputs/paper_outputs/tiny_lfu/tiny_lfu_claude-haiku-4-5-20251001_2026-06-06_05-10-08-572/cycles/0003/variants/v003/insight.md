# Variant v003 Insight

## Helped

- **hit-rate**: Periodic decay (every ~500k operations) makes the sketch responsive to workload phase changes by discounting stale access patterns, enabling slightly better admission decisions.
- **per-op-time**: Both get (112ns vs 132ns) and put (142ns vs 166ns) operations speed up significantly; periodic decay prevents unbounded count growth, keeping the sketch computationally lean and memory-efficient.

## Hurt

- **metadata-size**: TinyLfuCache struct grows by 8 bytes (152→160) to store the `op_count` counter for decay scheduling.

## Takeaway

Adding periodic frequency sketch decay improves both hit-rate and per-operation latency. The hit-rate gain is marginal but consistent with the hypothesis: decaying counts every ~500k operations makes the frequency sketch responsive to working-set shifts. The latency improvements are more significant, suggesting that periodic decay (replacing unbounded count growth with periodic resets) reduces computational overhead and improves memory access patterns—a co-benefit not explicitly targeted by the hypothesis.
