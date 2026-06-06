# Variant v001 Insight

## Helped

- hit-rate: Held at the parent's record — routing arena reads through the new `node`/`node_mut` helpers is a pure codegen change that touches no policy, so segment flags, `prot_len`, eviction order, and the protected-MRU fast path all behave byte-for-byte as before.
- metadata-size: Held at 24 bytes — the change added only two `#[inline]` accessors, leaving the `u32`-link `Node` layout and the 24-byte size class untouched.

## Hurt

- per-op-time: The intended win never materialized — replacing `self.nodes[idx]` with `unsafe { get_unchecked }` on the get/put hot paths regressed both ops at p50 and p99 instead of cutting them. The original bounds checks on these proven-in-bounds, sequentially-predicted arena accesses were already cheap or elided by the optimizer, so removing them bought no codegen and the helper-routed version came out measurably slower across the board.

## Takeaway

Bounds-check elision was the wrong lever: the arena accesses on get/put were not the latency bottleneck, so swapping them to `get_unchecked` traded real `unsafe` risk for a net latency regression rather than the predicted drop below the 27 ns champion. The per-op cost of this SLRU lives elsewhere (hashing, link surgery, cache-line traversal), not in redundant bounds checks the compiler was already handling.
