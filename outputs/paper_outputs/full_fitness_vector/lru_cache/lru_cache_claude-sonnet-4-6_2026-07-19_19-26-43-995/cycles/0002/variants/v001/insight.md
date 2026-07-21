# Variant v001 Insight

## Helped

- **hit-rate**: Layering a probationary FIFO + protected LRU (2Q) policy over the u32-sentinel layout shields frequently-accessed entries from cold-data eviction, recovering the same hit-rate lift that cycle/0001/v001 demonstrated but could not sustain alongside compact metadata.

## Hurt

- **metadata-size**: The `LruCache` struct grew from 88 to 120 bytes because two queue head/tail pairs (`prob_head`, `prob_tail`, `prot_head`, `prot_tail`) plus three bookkeeping counters (`prob_len`, `prot_len`, `prob_cap`) replaced the single `head`/`tail` pair; `Node` size is unchanged at 24 bytes so the per-entry summary metric held steady.

## Takeaway

The protected-bit-in-prev trick (`PROT_BIT = 0x8000_0000` with a revised `NULL = 0x7FFF_FFFF`) successfully encodes queue membership inside the existing `prev` field, keeping `Node` at 24 bytes while enabling full 2Q semantics. The hit-rate improvement confirms that the policy itself — not layout bloat — was responsible for the gain seen in cycle/0001/v001: the compact node layout means the same capacity fits in fewer cache lines, and the 2Q graduation logic then operates on those denser nodes without measurable per-op latency cost.
