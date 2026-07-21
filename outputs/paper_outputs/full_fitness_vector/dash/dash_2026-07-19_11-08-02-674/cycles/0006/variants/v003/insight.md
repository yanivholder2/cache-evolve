# Variant v003 Insight

## Helped

- hit-rate: The two-chance clock (`clock_evict_prepare`) combined with MRU-end insertion spares warm items — those that accumulated at least one hit via `lfu_counter` — from being evicted at the front of the bucket, so cold tail items on a Zipfian workload are displaced first.
- metadata-size: Dropping the `timestamp: Instant` field from `Item<K, V>` shrinks each item struct from 40 to 24 bytes, a 40 % reduction that directly compresses the working-set footprint.
- per-op-time: Eliminating `Instant::now()` from the eviction path removes system-clock reads on every eviction, and the smaller item struct improves cache-line utilisation during bucket traversal; both effects reduce median and tail latency for `get` and `put`.

## Hurt

*(no dimension moved unfavourably)*

## Takeaway

Replacing timestamp-based eviction with a two-chance clock that reads `lfu_counter` achieves a better hit rate than pure positional LRU while simultaneously reducing per-item memory and operation latency, because the clock's early-exit on the first cold item is cheap on Zipfian workloads and removing `Instant` eliminates the dominant source of overhead in the old `TimestampLRU` path. The three changes (timestamp removal, MRU-back insertion, clock eviction) are mutually reinforcing: MRU insertion ensures counters are meaningful when the clock inspects them, and the smaller struct makes the rotate-left scan less expensive.
