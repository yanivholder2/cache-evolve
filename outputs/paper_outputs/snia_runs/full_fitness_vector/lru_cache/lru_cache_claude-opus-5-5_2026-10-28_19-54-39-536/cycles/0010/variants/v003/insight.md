# Variant v003 Insight

## Helped

- per-op-time (get p99): The get tail dropped sharply. This is the one place where the denser link array plausibly pays off. Tail hits do relinks whose neighbours (`links[prev]`, `links[next]`, the segment tail and a demoted `prot_head`) are scattered, and 8 links fit in one cache line where only 2–3 full nodes did. So those scattered touches miss less often. The parent's insight suspected its own get p99 was partly noise or a hash-table side effect, so a re-run is needed before this gain is credited fully to the split.
- metadata-size: The reported value fell only because `Node` lost its `prev`/`next` fields. Those 8 bytes moved into `Link`, so total bytes per entry are unchanged at 24. This is an accounting artifact, not a real saving (the hypothesis says so too).

## Hurt

- per-op-time (get p50): The hypothesis's main prediction failed, and the get median got slightly worse. Every hit now loads two separate cache lines for the entry itself, `links[idx]` for the segment check and unlink and `nodes[idx]` for the returned value, where the parent used one co-located node. That extra line costs more at the median than the denser neighbour touches save.
- per-op-time (put p99): The put tail rose. A filling insert now pushes into two `Vec`s. An evicting insert writes `nodes[victim]` (key/value swap) and also touches `links[victim]` and its neighbours, so the worst-case put touches more distinct lines than the parent's single-node layout.
- metadata-size (LruCache header): The cache struct grew by one `Vec` header (pointer/cap/len) for the new `links` field.

## Takeaway

Splitting `prev`/`next` into a parallel `Vec<Link>` keeps the policy bit-identical, since hit rate is unchanged. But it does not lower get p50, because the median hit is dominated by the entry's own lines, and the split turns that one line into two. Neighbour density only shows up in the tail. Future layout work aimed at get p50 should keep the hit entry's link and value together, for example by shrinking the node, not by separating its fields. Any credit for the SoA split rests on the get-p99 gain, which has to survive a noise re-run.
