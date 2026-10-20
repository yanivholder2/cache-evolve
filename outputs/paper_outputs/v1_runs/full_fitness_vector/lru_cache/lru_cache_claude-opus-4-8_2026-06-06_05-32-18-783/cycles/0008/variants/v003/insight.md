# Variant v003 Insight

## Helped

- (none) — the ghost re-admission path did not lift any dimension over the parent. hit-rate, the dimension it was built to move, went the wrong way.

## Hurt

- hit-rate: admitting every returning ghost key *directly into protected* (`admit_protected` branch in `put`, `prot_len += 1` with overflow demote) overshot exactly as the hypothesis's refutation clause warned — proven-once returners flooded the protected segment and pushed its `demote_protected_lru` to evict genuinely hot keys, so hit-rate fell below the parent and under the ~0.729 confirm threshold.
- per-op-time: the new `ghost_set.remove` probe on every new-key `put` plus `record_ghost` bookkeeping (set insert + queue push/pop) on every eviction added work to the put path, raising put p50; get p50 also drifted up though `get` was untouched, consistent with the larger struct footprint hurting locality.

## Note

- metadata-size: the scored `Node<u64,u64>` held at 24 bytes as designed — the ghost `HashMap` + `VecDeque` live on `LruCache`, not `Node` — but the `LruCache<u64,u64>` footprint grew from 96 to 168 bytes for two extra collections that bought nothing.

## Takeaway

This settles the standing cross-cycle question negatively: on this short-reuse trace, smarter ghost-based re-admission does *not* reproduce the gated-ghost hit-rate gain. Treating a single cross-eviction return as proof of hotness and skipping probation pollutes the protected segment and evicts truly-hot keys, so reuse-distance admission costs both hit-rate and latency here — the parent's plain SLRU geometry remains the better champion.
