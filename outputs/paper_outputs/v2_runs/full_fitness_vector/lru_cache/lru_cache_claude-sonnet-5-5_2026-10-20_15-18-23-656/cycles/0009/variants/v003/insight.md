# Variant v003 Insight

## Helped

- metadata-size: No change. Node and struct sizes match the parent, as expected, since the edit only changes where a node is linked and adds no fields.

## Hurt

- hit-rate: Linking ghost-hit keys at the probation MRU end (or second position) made hit-rate worse than the parent. Returning keys now sit ahead of resident keys that have already been re-referenced, so they push those keys toward eviction. The parent's LRU-end insertion for all probation arrivals handled this better.
- per-op-time: `get` and `put` p50 and p99 are all somewhat slower than the parent, with `get` p99 moving most even though `get` was not edited. Part of this is likely measurement noise. The extra head-pointer handling on the ghost-hit `put` path cannot explain a `get` change.

## Takeaway

The middle path between cold-end insertion and direct protected promotion does not pay off here. A ghost hit is only weak evidence of reuse, and giving it a full probation stay costs more in displaced useful keys than it gains in retained returning keys. The ghost filter's benefit seems to come from keeping suspicious keys cheap to evict, not from rewarding the keys it remembers. Note that I could not read the source files in this session, so this interpretation rests on the hypothesis text and the fitness vectors.
