# Variant v003 Insight

## Helped

- hit-rate: The doorkeeper refuses a new key's first sighting at capacity, so one-hit keys no longer evict residents; the hit rate rose over the parent on workload-a at size 2048.
- per-op-time: The median `put` dropped below the parent's, likely because refused first-sighting keys skip the victim selection, unlink, and map remove/insert work, which more than pays for the new hash on the miss path.

## Hurt

- per-op-time: The `put` tail moved slightly up, which is the cost of hashing every new-key miss at capacity through `door_slot` before any admission decision.
- metadata-size: The summary still reports the 24-byte node, but `LruCache` gains a `Vec<u8>` header and a `door_ops` counter, and the heap doorkeeper table (next power of two above twice capacity, 4096 bytes at capacity 2048) is outside the measured dimension.

## Takeaway

A first-sighting filter at the admission point trades a small, bounded hash cost on new-key misses for fewer evictions of resident keys under a scan-heavy trace. This variant does not separate rejected one-hit keys from the periodic aging reset (every 10 × capacity consultations), and counters are shared by hash slot, so a colliding one-hit key can be admitted early; a variant with aging disabled or a larger table would be needed to tell these apart.
