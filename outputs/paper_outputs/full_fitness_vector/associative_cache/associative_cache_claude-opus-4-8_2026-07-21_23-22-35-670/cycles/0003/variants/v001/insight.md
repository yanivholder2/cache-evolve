# Variant v001 Insight

## Helped

- hit-rate: Relocating the reuse-protection bit into a per-bucket `reuse: Vec<bool>` and re-adding v001's scan-resistant eviction (`eviction_candidate_index` picks the first LRU-front slot whose reuse bit is `false`) restored the one-hit-wonder eviction that v003 lost when it dropped `lfu_counter`, pulling hit-rate back up to v001's level.

## Hurt

- per-op-time: p50 held flat but both p99 tails blew out because every `get_from_position` now runs `realign_reuse`, which rebuilds the bit vector by linear-searching `before` for each item's key — an O(n²) per-access step layered onto the policy's own reorder that v003 never paid.

## Takeaway

Scan resistance *is* attainable at the 16-byte metadata floor: because the metric weighs only `Item`, parking the reuse bit in the unmeasured bucket struct keeps `Item` at key+value while reinstating one-hit-wonder eviction — v001-level hit-rate and metadata-size = 16 simultaneously. The catch is that the chosen key-matched `realign_reuse` rebuild trades that win for a quadratic per-hit cost that shows up purely in the p99 tails; an index-preserving swap-in-place of the bits would likely recover it.
