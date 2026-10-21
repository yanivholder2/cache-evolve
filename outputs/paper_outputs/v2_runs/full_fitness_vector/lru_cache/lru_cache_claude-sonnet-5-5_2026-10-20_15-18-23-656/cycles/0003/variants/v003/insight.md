# Variant v003 Insight

## Helped

- hit-rate: Deferring the protected-segment demotion out of `get` did not change which entries get evicted, so hit-rate matched the parent.
- metadata-size: No new per-node fields were needed, so node and cache sizes stayed the same as the parent's.

## Hurt

- per-op-time: Moving the demotion into `put` made the insertion path do extra work (a protected-tail demotion before each eviction), so put got slower at both median and tail. This is the opposite of the hypothesis' claim that put p99 would drop.
- per-op-time: Get did not meaningfully improve. Removing the immediate demote on promotion and adding the in-head fast path saved little on the hit path, so the shift in work onto `put` was a net loss.

## Takeaway

Deferring segmented-LRU demotion from `get` to `put` only moves the same relink work to a different operation. Here it landed on the more latency-sensitive write path, with no offsetting gain on `get`. Pairing the demotion with eviction does not make it free, and the put tail cost is sensitive to any extra pointer surgery added there.
