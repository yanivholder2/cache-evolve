# Variant v003 Insight

## Helped

- metadata-size: dropping the 16-byte `Instant timestamp` and narrowing `lfu_counter` from `usize` to `u8` shrank `Item<u64,u64>` to its key+value+1-byte-counter footprint (aligned to 24).
- per-op-time: removing the `timestamp` field killed the `Instant::now()` write on every `get`/`put`, so both get and put p50/p99 fell sharply.

## Hurt

- hit-rate: unchanged — the eval policy is `ClassicLRU`, which is purely positional and never read `timestamp` or `lfu_counter`, so collapsing those fields left eviction order untouched.

## Takeaway

The baseline `Item` paid for an `Instant` and a word-sized LFU counter that the active `ClassicLRU` policy never consults; stripping them to a single byte was free on hit-rate and also a latency win, because the timestamp's hidden cost was the `Instant::now()` syscall on the hot path, not just its bytes. Reimplementing `TimestampLRU` as positional recency and making the `Lfu` counter saturate kept the JNI policy contract intact while removing all dead reads of the deleted fields.
