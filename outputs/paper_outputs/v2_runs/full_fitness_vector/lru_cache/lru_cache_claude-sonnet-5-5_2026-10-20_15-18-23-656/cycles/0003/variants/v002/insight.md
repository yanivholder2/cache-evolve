# Variant v002 Insight

## Helped

- hit-rate: Splitting the list into probation and protected segments, with promotion only on a second hit, keeps one-touch scan entries from displacing the reused working set. This gave a large gain over the parent's plain LRU, and it landed at the hypothesis target.
- metadata-size (per-node): Putting the segment flag in bit 31 of the `next` u32 left `Node` the same size as the parent. Per-entry overhead did not grow.

## Hurt

- per-op-time: Every `get` hit in probation now unlinks the node and relinks it at the protected head. Protected overflow also demotes the protected tail. The parent's head-touch early return now covers only the protected head. Together these made both get and put noticeably slower than the parent. This was well off the "near 27" the hypothesis expected.
- metadata-size (container): `LruCache` grew because it holds a second head/tail pair and the segment bookkeeping. The `Node` summary metric hides this cost.

## Takeaway

Segmented LRU bought its hit-rate gain at the price of extra pointer surgery on the hot path. Packing the flag into the link kept per-node size flat, but it could not remove the promotion and demotion work. The hypothesis was right that per-node metadata would not grow and wrong that latency would stay flat. Any further scan-resistance work needs a cheaper promotion path, such as skipping relinks for nodes already near the head.
