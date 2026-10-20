# Variant v003 Insight

## Helped

_None. The single measured dimension, hit-rate, did not move from the parent v002, so the insert-count trigger produced no measurable gain on workload-a._

## Hurt

_None measured. The fitness vector for v003 contains only `hit-rate`, which is unchanged from the parent, so no regression is visible. `metadata-size` and `per-op-time` were not evaluated and cannot be judged against v002._

## Takeaway

H03 moved the AgedLfu halving from a per-bucket hit counter to a cache-wide insert counter with period `max(64, capacity / 8)`, so that decay should fire many times across a 2048-slot run. Hit-rate came back identical to v002 at the reported precision, which means the tested configuration did not see any change in eviction order from the more frequent halving. This is the same null result as v002 and does not separate the two explanations the hypothesis raised: the harness may not be selecting the `AgedLfu` policy, or the decay never reorders the minimum-counter victim scan on this trace. Before the aging mechanism is judged again, the eval path should confirm the policy is active and log how many halvings fire.
