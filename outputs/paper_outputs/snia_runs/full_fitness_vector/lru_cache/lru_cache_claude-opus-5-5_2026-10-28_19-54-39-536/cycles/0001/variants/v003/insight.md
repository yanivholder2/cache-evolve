# Variant v003 Insight

## Helped

- metadata-size: Swapping the two `Option<usize>` links for `u32` indices with a `u32::MAX` sentinel removes the discriminant padding and halves pointer width. This cuts `Node<u64,u64>` in half, to exactly the size the hypothesis predicted. The same change to `head`/`tail` also shrinks the `LruCache` struct header.
- per-op-time: `put` latency improved slightly at both p50 and p99. A plausible cause is that the smaller nodes pack more entries per cache line during eviction and relinking, which touch neighbouring nodes. This is only a secondary effect; the summary dimension (get p50) did not move.

## Hurt

- per-op-time: `get` p99 rose a little. The likely cause is the extra `u32`→`usize` casts on each index, or plain measurement noise. It is too small to call a regression, and get p50 is unchanged.

## Takeaway

How the links are stored has no effect on the policy. Eviction order is unchanged, so the hit rate on every MSR trace and size is identical to the baseline. That makes this a free win on metadata-size and a safe base to build on. Because `try_new` refuses any capacity at or above `u32::MAX`, the 32-bit indices give up nothing for realistic cache sizes. Halving node size did not noticeably speed up the hot `get` path, so later variants should not rely on node compaction for speed.
