# Variant v002 Insight

## Helped

(no dimension improved)

## Hurt

- **hit-rate**: The 2-candidate frequency-guided eviction within the probationary queue produced no measurable improvement; at `prob_cap = capacity/16` (128 slots at size 2048), probationary items are evicted so quickly—and the one-hit graduation rule removes re-accessed items immediately—that the 64-bucket flat table almost never accumulates distinguishable frequency signal between a key's insertion and its eviction.
- **metadata-size**: Adding `freq_table: [u8; 64]` to `LruCache<K, V>` expanded the struct by 64 bytes (one full cache line), a fixed per-instance overhead that accrues regardless of workload, with no corresponding policy benefit observed here.

## Takeaway

Frequency hints at eviction time are only useful when probationary items actually receive re-accesses before being expelled, but the current `capacity/16` probationary window is too narrow for the flat frequency table to see meaningful signal: most probationary slots turn over without any hit, making the 2-candidate comparison a coin flip. This variant shows that frequency-guided eviction and one-hit graduation are complementary only when the probationary quota is large enough for re-accesses to register—at 93.75% protected, the probationary queue is already so thin that the frequency table adds struct size without changing which item gets evicted.
