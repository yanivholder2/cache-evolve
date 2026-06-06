# Variant v002 Insight

## Hurt

- hit-rate: Relaxing the admission threshold from `>` to `>= victim_freq - 1` admitted items with frequencies only marginally above the victim, but many of these marginal admissions were poor decisions that displaced more valuable hot items, reducing overall hit-rate.
- per-op-time (get p99): The increased admission rate raised the tail latency of get operations, likely due to increased cache churn and contention under this more permissive policy.
- per-op-time (put p99): Tail latency of put operations worsened, suggesting the added complexity of evaluating and accepting marginally-hot items introduced occasional spikes in operation cost.

## Helped

- per-op-time (get p50): Median get latency improved slightly, possibly because the reduced selectivity of admission decisions lowered decision complexity in the common case.
- per-op-time (put p50): Median put latency improved, likely from the same mechanism—simpler admission logic path when items are evaluated at the threshold boundary.

## Takeaway

Relaxing the admission threshold to a range-based condition (`>= victim_freq - 1`) proved counterproductive: it admitted too many low-value items, hurting hit-rate despite the hypothesis's intuition that a middle ground between strict and permissive policies would yield better results. The baseline's strict `>` condition is closer to optimal for this workload. The modest gains in median latency do not offset the loss in cache effectiveness.
