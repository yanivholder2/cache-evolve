# Variant v002 Insight

## Helped

- metadata-size: The `freq: u8` counter takes the padding slot that `visited: bool` used, so `Node<u64,u64>` and the cache struct stay the same size. The extra frequency state cost no space, which is what the hypothesis intended.

## Hurt

- hit-rate: The 2-level counter did not beat SIEVE's single visited bit and finished marginally below the parent, so the hypothesis failed. When a key gets a second hit, it earns one more pass of the hand. During that pass it holds a slot that a scan-resistant single bit would have freed for a fresher key. On this trace, that extra frequency credit is worth about as much as the space it ties up.
- per-op-time: `put` tail latency is the main regression, and `get` got slightly worse too. When every node has saturated, `find_victim` can now sweep up to two full revolutions instead of one, so the occasional slow insert gets longer. On the hit path, the old plain store of `visited = true` is now a load-compare-add read-modify-write on `freq`.

## Takeaway

Adding GCLOCK-style multi-level credit to SIEVE did not carry over GCLOCK's gain over CLOCK. SIEVE's quick demotion of new keys already captures the reuse signal that a counter adds over plain CLOCK, so the second level brings no new hit-rate. Its only effect is a longer worst-case sweep, at no saving in metadata. Further hit-rate gains on this trace will probably need a different signal than how many times a key was hit while it sat in the queue, such as ghost/history entries for evicted keys.
