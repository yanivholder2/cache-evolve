# Variant v002 Insight

## Helped

- None. Hit-rate on workload-a fell relative to the parent, and get latency did not improve.

## Hurt

- hit-rate: Halving every remaining slot's access counter after each eviction erodes the frequency signal that the parent's lowest-counter scan depends on, so residents that were hit repeatedly but not in the last few evictions lose their protection and are chosen as victims sooner. This is the opposite of the retention effect the parent's insight credited to the counter.
- per-op-time (put): The decay pass runs inside the eviction branch of `evict_one()`, after the victim is chosen, so every put that lands in a full bucket now also walks and rewrites the counters of the rest of the bucket. Put is the only operation that touches this code, and its tail latency moved the most of any measured value.

## Takeaway

On workload-a, decaying access counters makes victim selection less sticky and costs hit-rate, which suggests the parent's never-decaying counter is doing useful retention work rather than merely shielding stale residents. The result is limited to workload-a: the second-trace check the hypothesis asked for was not run, so this variant cannot show whether the loss generalizes or whether a milder decay schedule would behave differently.
