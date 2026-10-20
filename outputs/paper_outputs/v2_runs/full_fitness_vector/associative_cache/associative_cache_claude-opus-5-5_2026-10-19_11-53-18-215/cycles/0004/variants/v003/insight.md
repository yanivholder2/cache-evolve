# Variant v003 Insight

## Helped

- hit-rate: Cold (ghost-miss) keys now enter at probation index `len0 / 2` instead of probation MRU. A one-shot scan or loop key therefore gets about half of v001's probation lifetime before it reaches `items[0]`. Keys that come back soon after eviction are recognised through the per-bucket fingerprint ring and re-enter at probation MRU, so they get the full window to earn promotion. The gain is real but modest. The likely limit is the ring size, `max_size / 2` fingerprints per bucket, which only remembers the most recent probation evictions. Keys whose reuse distance is longer than that are still treated as cold.
- per-op-time (put p50): The common miss path on a full bucket with a cold key used to be `remove_at(0)` (which shifts the whole Vec, protected segment included) followed by `insert(len0, ..)` (which shifts the protected segment again). It is now one `mem::replace` plus `rotate_left(1)` over `items[0..=mid]`, touching only the lower half of probation. This is the move-cost saving the hypothesis predicted.

## Hurt

- per-op-time (put p99): Every new-key put now costs extra work that v001 did not do:
  - It computes a second `hash(&key)` to get the fingerprint.
  - It scans the whole ghost ring linearly in `take_ghost`.
  - It writes to the ring on each probation eviction.

  The ghost-hit and `len0 == 0` branches also keep v001's two-memmove `remove_at(0)` + `insert(len0)` sequence, and now carry the fingerprint overhead on top. This puts the slowest puts above v001's tail. Get p50 is the dimension summary, and the get path did not change, so the summary did not move. Any change in get p99 cannot be traced to this diff.

## Takeaway

Using ghost history only to choose where a new key enters probation (middle for cold keys, MRU for ghost hits) is a cheap, safe way to resist scans on the merged-bucket SLRU. Protected items are never displaced, and the common-case put gets a shorter move. The cost moves into the put tail: the extra hash and the linear ghost scan run on every miss. Look at those first before growing the ring to capture longer reuse distances. Separately, the ring adds per-bucket memory: a `Vec` header plus `max_size / 2` × `u16`. The metadata-size dimension does not see it, because that dimension measures only `Item` and the cache handle.
