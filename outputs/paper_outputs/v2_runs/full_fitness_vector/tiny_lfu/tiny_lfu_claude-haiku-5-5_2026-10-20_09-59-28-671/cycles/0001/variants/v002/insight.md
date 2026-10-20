# Variant v002 Insight

## Helped

- per-op-time: `get()` returns early when `idx` is already `self.tail`, skipping the `unlink`/`push_back` pointer writes on repeated hits to the MRU key, which is the common case on a skewed trace.
- per-op-time: The same tail guard in the update branch of `put()` skips the unlink/push_back pair when overwriting the most recently used key, so `put` gets the same saving on that path.

## Hurt

## Takeaway

The guard only removes list relinks that would have left the tail unchanged, so the LRU order and the sketch are untouched and the hit rate is structurally identical to the parent. The gain is purely constant-factor work on MRU-repeat accesses, so it scales with how often the workload re-hits its newest key rather than with cache size or policy quality.
