# Variant v002 Insight

## Helped

- put: p50 and tail latency improved slightly, plausibly because most new keys are now appended at the window tail and the main-segment admission comparison runs only when the window overflows. This is a small gain and I did not read the source to confirm it.

## Hurt

- hit-rate: Essentially no change. The ~1% window gives a new key very little time to build frequency before the sketch comparison, so admission behaves almost as it did in the baseline.
- metadata-size: The added `seg` field grew `Node` and padded it, and the extra `win_head`, `win_tail` and `win_len` fields grew the cache struct, with no matching benefit.
- per-op-time: `get` got slower because every hit must first work out which segment (window or main) the node is in, then unlink and relink it in that segment's list.

## Takeaway

A W-TinyLFU-style window of about 1% of capacity did not change admission outcomes on this trace. It cost extra per-node and per-cache metadata and added a segment branch to every `get`. Splitting the LRU into segments only helps if the window is large enough to matter, or if the trace has the scan or recency pattern the hypothesis targeted. Neither was true here.
