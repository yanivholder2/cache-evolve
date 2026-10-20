# Variant v001 Insight

## Helped

- per-op-time: Replacing the hash-check plus linear `small`/`main` scan with a single `resident.get_mut` probe removes the O(n) walk on hits. The `get` and `put` p99 tails shrank the most, because the worst cases were the long scans. The p50 moved only slightly, since most hits were already found near the front or with little scanning.

## Hurt

- metadata-size: Nothing moved. The new `Slot { value, freq, in_main }` map entry and key-only queues apparently cost about the same as the old entries, so there was no gain or loss.

## Takeaway

Collapsing the linear scan into one keyed lookup, with queues holding only keys, delivered a mostly tail-latency win while leaving eviction behavior bit-identical. That is exactly what the hypothesis required: hit-rate is unchanged, so the refactor preserved policy semantics. The scan was a tail-latency cost, not a median cost, at this cache size (2048). Note: I could not read the overlay sources in this session, so this analysis rests on the hypothesis and the fitness vectors.
