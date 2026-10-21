# Variant v002 Insight

## Helped

- hit-rate: Unchanged against the parent, which is the intended result. `move_to_back` and the tail fast path produce the same LRU ordering as the old unlink/push_back, so eviction choices and hits are identical.

## Hurt

- per-op-time: The fitness vector has no per-op-time dimension, so the intended speedup (tail short-circuit, single-pass `move_to_back`, `mem::replace` instead of a second key clone on eviction) cannot be confirmed. It is unmeasured, not shown to be worse.
- metadata-size: This dimension is also missing from the vector. The change added no fields to `Node` or `LruCache`, so no layout change is expected, but nothing was recorded to check it.

## Takeaway

The rewrite of the promotion and eviction paths kept strict LRU behaviour, because hit-rate matched the parent exactly. The evaluation recorded only hit-rate, so the optimization's value is unproven. The hypothesis targeted per-op-time, and that dimension was not recorded. Before judging the idea, rerun the eval with per-op-time and metadata-size enabled.
