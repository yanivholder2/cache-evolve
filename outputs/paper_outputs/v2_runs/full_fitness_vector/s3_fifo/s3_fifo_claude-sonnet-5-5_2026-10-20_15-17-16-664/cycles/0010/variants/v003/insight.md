# Variant v003 Insight

## Helped

- hit-rate: No movement, as intended. The hypothesis held policy, sizes and eviction order fixed, so the hit rate is unchanged.
- per-op-time (get/put p50): No movement. Hashing a u64 key is already cheap, so reusing the hash saved nothing measurable at the median.

## Hurt

- metadata-size: `Entry` grew by one 8-byte word. This is most likely a stored or carried hash, which is the usual way to reuse a hash across the resident, ghost and insert probes. It is a direct cost with no measured benefit.
- per-op-time (put p99): The `put` tail got markedly worse. The pre-hashed or raw-entry path may have added overhead on the slow path (rehash on growth, eviction, ghost handling). It may also be run-to-run noise on a tail metric. Either way, the intended tail reduction did not appear.

## Takeaway

Hash reuse in `put` did not pay off. The `put` tail is not dominated by hash computation of small integer keys, so the lever targeted the wrong cost. It also cost an `Entry` field and a worse `put` tail. This change should not be kept unless a later variant shows a real tail win. I could not read the overlay sources in this session, so the mechanism is inferred from the hypothesis and the fitness vectors.
