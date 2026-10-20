# Variant v001 Insight

## Helped

- hit-rate: Not a gain, but the invariant held. Keying the map by `key_hash` and adding the "reject on hash collision" policy left the hit rate identical bit for bit. This confirms that the sketch order, CLOCK sweep, hand movement and strict `>` duel were all preserved, and that 64-bit hash collisions never fire at capacity 2048.
- metadata-size: Not a gain, but the constraint held. `Node` is still `{ key, value }` at 16 bytes and no fields were added to `TinyLfuCache`. The map entry is still a `u64` key plus a `u32` slot, so swapping `K` for its hash costs nothing for `u64` keys.

## Hurt

- per-op-time: The "hash once" saving was close to zero because the map-side work it removed was `FxLikeHasher` on a single `u64`: one rotate, one xor and one multiply. Meanwhile `get` gained an explicit `nodes[idx].key != *key` check. That check is a dependent load plus a branch, and it has to resolve before `set_ref` and the value return. So the typical `get` and `put` both got slightly slower, not faster.
- per-op-time: The `get` p99 regressed much more than the p50. Nothing in the diff adds an unbounded path to `get`, so the change can't account for that tail by itself. One possible link is that the identity hasher now puts the low bits of `h` in charge of both SwissTable bucket selection and sketch row-0 column selection, though this is unconfirmed. Otherwise it may be measurement noise, so re-run the benchmark before treating it as real.

## Takeaway

Identity-keying the map by the precomputed `fmix64` hash works and keeps hit rate unchanged. But for `u64` keys the duplicate hashing it removes was already almost free, so the collision-guard compare it adds to every hit cancels the savings or worse. The latency left on the `get` path is the sketch's 4-row update and the map probe's memory access, not the hash calculation. Future attempts should cut memory traffic, for example by fusing or shrinking the sketch rows, rather than reducing hash calls.
