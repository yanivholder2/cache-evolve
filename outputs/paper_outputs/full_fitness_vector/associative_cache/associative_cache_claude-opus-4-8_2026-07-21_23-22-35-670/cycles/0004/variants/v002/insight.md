# Variant v002 Insight

## Helped

- metadata-size: The `reuse: Vec<bool>` protection state was parked on the bucket struct rather than on `Item`, so the one-hit-wonder bit was added without the eval weighing it — `Item` stayed at the key+value floor, unchanged from the parent.

## Hurt

- hit-rate: Stacking an explicit reuse bit on top of v003's front admission did nothing — under ClassicLRU a hit already moves the item to the back via `get_from_position` while fresh misses sit at index 0, so the frontmost slot is *always* the probationary one and the overridden `eviction_candidate_index` (first `reuse == false`) resolves to the same index the parent's default already evicted. The hypothesis's target of beating 0.7267 was not met; hit-rate is identical to the parent.
- per-op-time: The extra `remove`/`insert` on `reuse` in `get_from_position` and the linear `reuse` scan in `eviction_candidate_index` are pure added work that bought no accuracy; latency held only because the 100 ns measurement grain hides the cost, not because the cost is zero.

## Takeaway

Under ClassicLRU, front admission plus move-to-back-on-hit *already* sorts probationary items (front) from protected items (back), so an explicit `reuse` bit re-encodes information the item ordering carries — the two scan-resistance levers are the same lever expressed twice, not compounding ones. The variant is behaviorally indistinguishable from its parent, confirming that reuse protection adds value only when it can override a policy that does *not* positionally segregate reused items.
