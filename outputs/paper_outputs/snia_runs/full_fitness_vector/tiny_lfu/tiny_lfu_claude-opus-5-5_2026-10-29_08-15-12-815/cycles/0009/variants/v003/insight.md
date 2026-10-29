# Variant v003 Insight

## Helped

- metadata-size (Node<u64,u64>, summary): The reference bit lives in a separate `ref_bits` bitset and is never packed into `prev`/`next`, so `Node` stays 24 bytes. Unlike cycle 0008 v003, every link store stays a blind store. This is the only part of the hypothesis's design goal that held.

## Hurt

- hit-rate (all four cells, summary): Every cell fell, and the loss is well beyond the 0.001 tolerance the hypothesis set. The protected segment now keeps promotion order, not recency order. On overflow the oldest-promoted head gets at most one second chance. After that the next head is demoted unconditionally, and its bit is cleared, even if it was just referenced. So hot protected entries are regularly pushed back into probation, where they have to win the admission contest against window candidates. The larger the protected segment, the further this FIFO-like order drifts from LRU, which matches the bigger drop in the 131072 cells, web_0 especially.
- per-op-time (get p50/p99, put p50/p99, summary): All four latency statistics got worse, including put, which this change barely touches. The intended saving was small to begin with. The parent already skipped the relink when a protected hit landed on the tail, and the new path still does a read-modify-write on a separate bitset line. At the same time, the promotion path got heavier: a referenced head is rotated before the demotion, so up to three unlink/push_back pairs run instead of two. Hot keys that were wrongly demoted also get re-hit in probation and promoted again, so this expensive path runs more often. Put adds `clear_ref` calls on every slot reuse and in `evict_one`. A single run cannot rule out noise. But all four statistics moved the same way, which argues against noise explaining the result.
- metadata-size (TinyLfuCache<u64,u64>): The cache struct grew by the `Vec` header for `ref_bits`. There is also about one bit per entry of heap that this dimension does not report. The headline per-node summary did not change.

## Takeaway

Replacing move-to-tail on protected hits with a CLOCK-style reference bit failed on both axes. The hit-rate loss comes from the policy change itself: a single second chance, followed by unconditional demotion, is a poor stand-in for protected-segment LRU. Moving the bit out of `Node` did fix cycle 0008's RMW problem. The other fix, capping rotations at one, kept rotation cost out of promotion only by giving up recency information. Get p99 got worse rather than better, so the protected relink is not where the get tail comes from. Later variants should keep the parent's protected move-to-tail and look for the tail cost elsewhere, such as the inlined sketch update or the map probe.
