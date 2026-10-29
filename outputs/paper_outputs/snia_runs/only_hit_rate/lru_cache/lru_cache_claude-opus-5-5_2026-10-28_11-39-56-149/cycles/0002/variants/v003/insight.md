# Variant v003 Insight

## Helped

- hit-rate: On hm_0, at both capacities, LIRS's non-resident HIR history in stack S lets a block evicted from the small resident-HIR queue come back as LIR on its next reference. That keeps long-reuse-distance blocks that plain LRU would cycle out. The gain is largest at the small capacity, which is where LRU's recency-only window is shortest.
- hit-rate: On web_0 at the large capacity, LIRS beats LRU by a modest margin. With more LIR slots, the stable reused set is protected from one-shot traffic, which can only pass through Q.

## Hurt

- hit-rate: On web_0 at the small capacity, LIRS does slightly worse than LRU. With `hir_cap = capacity / 100`, every new block gets only about 1% of the cache as its probation window, and the non-resident history is cut off at `2 * capacity`. Blocks whose reuse falls just past that short window, but within LRU's full-cache window, now miss where LRU would have hit.

## Takeaway

Using inter-reference recency (LIRS) instead of plain recency mostly paid off. The gains are largest on hm_0, where reuse distances are long and the non-resident history in S detects reuse that LRU misses. The one regression, web_0 at the small capacity, points at the fixed 1% HIR split. When the whole cache is small, a 1% probation queue is too narrow for a recency-friendly trace. The HIR fraction (or the size of the non-resident bound) is the natural parameter to tune next, not the LIRS mechanism itself. v003's vector reports only hit-rate, so this variant says nothing about metadata-size or per-op-time, even though its node now carries three link pairs.
