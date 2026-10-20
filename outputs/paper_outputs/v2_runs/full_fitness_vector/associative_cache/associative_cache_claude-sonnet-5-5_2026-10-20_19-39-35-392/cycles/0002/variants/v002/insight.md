# Variant v002 Insight

## Helped

- hit-rate: Evicting the lowest-frequency slot (LRU tie-break) keeps repeatedly hit items alive through one-hit scan traffic that plain LRU would let flush them, giving a small gain on workload-a.

## Hurt

- per-op-time: A hit now updates the counter in addition to move-to-back, and a put into a full bucket scans the whole bucket for the minimum counter plus periodic aging. Both get and put got slightly slower.

## Takeaway

A 2-bit saturating frequency signal gives a real but small hit-rate gain on this trace. The u8 counter fit into existing struct padding, so metadata-size didn't change. The price is a small, consistent per-op latency increase from the extra bookkeeping on every access. The modest gain suggests the baseline's per-bucket recency order already captures most of what this trace offers, and frequency adds only marginal information at this bucket size.
