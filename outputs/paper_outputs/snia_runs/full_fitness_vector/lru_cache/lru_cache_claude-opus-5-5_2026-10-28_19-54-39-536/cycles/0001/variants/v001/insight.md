# Variant v001 Insight

## Helped

- hit-rate: Sending new keys to a 20% probation segment and promoting them only on a second hit stops one-touch block scans from pushing reused blocks out, so hit rate went up on both MSR traces. The gain was clearly larger at the 131072 size than at 16384. At the small size, probation holds only about 3K slots, so many blocks that would be reused get evicted before their second touch can promote them.

## Hurt

- metadata-size: The hypothesis assumed padding would absorb the new `protected: bool`, but it did not. `Node<u64,u64>` was already a padding-free 48 bytes (key, value and two 16-byte `Option<usize>` links), so one extra byte rounds the node up a full 8-byte word. `LruCache` also grew because it now carries two head/tail pairs plus `prob_len`, `prot_len` and `protected_cap`.
- per-op-time: Both `unlink` and `push_back` now branch on the node's segment to pick a head/tail and a length counter. `on_hit` makes one extra unlink/push_back call, and a promotion that overflows the protected segment makes a second pair to demote the protected LRU. Together these raised get and put latency at both p50 and p99.

## Takeaway

SLRU's frequency filter improves hit rate on these block-I/O traces, but only modestly, and most of the benefit appears at the larger cache size. The fixed 20/80 split leaves the small cache's probation segment too short to catch reuse. The cheap win the hypothesis predicted for metadata-size was wrong: the segment flag added a full word per node. A follow-up should store the flag in space the node already has, such as a sentinel or high bit in `prev`/`next`, or switch to `u32` arena indices so the extra field fits without growing the node.
