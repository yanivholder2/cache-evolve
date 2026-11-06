# Variant v002 Insight

## Helped

- hit-rate: Replacing exact LRU with CLOCK did not cost hit rate. It went up slightly overall, and the gain was clearest on `hm_0` at both cache sizes and on `web_0` at the small size. New keys enter with `referenced = false` while hit keys get a second chance, so blocks that are touched only once get evicted ahead of blocks that were reused. On these MSR traces that rough frequency filter works at least as well as strict recency ordering.

## Hurt

- hit-rate: `web_0` at the large size (131072) dropped slightly. When the ring is large, the hand can go a long time without sweeping, so many reference bits stay set. Victim choice then falls back toward FIFO order, which is coarser than true LRU on a trace where recency matters most at that size.

## Takeaway

This variant shows that CLOCK's single-bit "second chance" can stand in for an exact LRU list on these block traces without hurting hit rate, and it gives a small scan-filtering gain that LRU lacks. The hypothesis was mainly about cost: `get` writes one byte instead of relinking up to four arena nodes, and dropping `prev`/`next` shrinks `Node<u64,u64>` to 24 bytes, which the in-tree `node_is_compact` test checks. However, this variant's `fitness-vector.json` contains only the `hit-rate` dimension, so the `per-op-time` and `metadata-size` gains have not been measured yet. Those two dimensions need an eval before this cost-vs-hit-rate trade-off can count as confirmed.
