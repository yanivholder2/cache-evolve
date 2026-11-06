# Variant v003 Insight

## Helped

- hit-rate: Replacing strict LRU with the 20/80 segmented policy improved every MSR trace at both sizes. New one-hit keys now enter probation and are evicted from there, so re-referenced keys in the protected segment survive scans. The summary lands exactly on the figure the hypothesis predicted from v001, which shows the bit-packed port reproduces v001's eviction order.
- metadata-size: `Node<u64,u64>` stays at 24 bytes. Bit 31 of `prev` holds the segment flag, so SLRU adds no per-node cost. This is the core claim of the hypothesis. It works because `try_new` now rejects capacities of `IDX_MASK` (2³¹−1) or more, which frees bit 31 at a cost far above any traced size.
- per-op-time: The expected masking penalty never appeared, and get p50 actually improved slightly. The `& IDX_MASK` and flag test are single ALU ops on a field already loaded. A higher hit rate also means more gets return through the hit path rather than the miss path, so the accessor indirection costs nothing measurable on the summary metric.

## Hurt

- per-op-time: Put p50 and p99 both rose noticeably, and get p99 rose a little. The cause is extra work in the SLRU code: `on_hit` branches three ways on the segment, a promotion can trigger a second `unlink`/`push_back` to demote the protected LRU, every link op now picks one of two head/tail pairs, and eviction resets the reused slot's flag. Puts on existing keys go through the same `on_hit`, so put carries most of this cost.
- metadata-size (secondary, `LruCache` struct): The cache header grew. It now holds four `u32` head/tail fields plus `prob_len`, `prot_len` and `protected_cap`, where the parent had only `head`/`tail`. This is a fixed per-cache cost, not per-entry, so the summary dimension does not count it.

## Takeaway

SLRU's hit-rate gain and a compact 24-byte node can coexist. Storing the segment flag in the high bit of the `u32` `prev` link delivers the full policy benefit at zero per-node cost and no measurable get-p50 cost. The remaining price is in put, from the promote/demote double relink and the per-segment branching. A future variant should target that path, not node layout or masking.
