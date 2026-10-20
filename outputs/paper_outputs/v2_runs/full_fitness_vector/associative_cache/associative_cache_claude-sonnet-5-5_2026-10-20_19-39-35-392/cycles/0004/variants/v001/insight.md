# Variant v001 Insight

## Helped

- metadata-size: Unchanged versus the parent, so the ghost fingerprint was kept within existing padding/fields rather than growing `Item` or the cache struct. The "few bytes per bucket" budget was respected.

## Hurt

- hit-rate: Slightly lower than the parent. Ghost admission demoted new keys to the LRU end unless their fingerprint matched the last-evicted key. A one-entry, 8-bit ghost per bucket remembers too little history to recognise genuinely reused keys, and fingerprint collisions add noise. Reusable keys were therefore penalised along with scan items.
- per-op-time: The put path got slower, at both median and tail, from the extra fingerprint compare and the ghost update on every eviction. The get path stayed at the same median and its tail did not improve, so the summary (driven by get) did not move.

## Takeaway

A single-entry per-bucket ghost is too weak a signal to separate one-hit-wonders from reusable keys on this workload. It costs put-path work and slightly hurts hit rate without offsetting gains. Any admission filter would need deeper or more selective history, or a stronger scan signal, to beat the parent's plain insertion policy.
