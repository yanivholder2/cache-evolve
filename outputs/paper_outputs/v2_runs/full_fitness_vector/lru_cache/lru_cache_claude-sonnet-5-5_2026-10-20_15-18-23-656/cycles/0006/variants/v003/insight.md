# Variant v003 Insight

## Helped

- metadata-size: unchanged; the change only altered which list `put` relinks into, so the node layout and struct sizes stayed the same.

## Hurt

- per-op-time: get p50/p99 are slightly worse than the parent, and put p50 is flat. `get` code was not touched, so this is most likely measurement noise or code-layout effects, not a real cost. The tail early return did not make `put` measurably faster.

## Takeaway

Replacing `promote` with `touch_same_segment` on the existing-key branch of `put` did not move hit-rate at all. The workload-a trace apparently rarely or never overwrites a resident key, or the overwrites it does contain make no difference to which entries reach the protected segment. Hit-rate here is decided by the `get` and miss-insert paths, so further work on `put`-update semantics is unlikely to pay off on this trace.
