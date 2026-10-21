# Variant v003 Insight

## Helped

- per-op-time: The parent ran SipHash once per sketch row (4 per increment, 4 more per estimate) plus a SipHash `HashMap` lookup. v003 hashes each key once with an FxHash-style mixer and an fmix64 finaliser, then gets all four row indices from that hash by double hashing (`h1 + r*h2`) over one flat `Vec<u32>`. This cuts out most of the hashing work and the per-row pointer lookup. Both get and put improved at p50 and p99.
- per-op-time (put tail): The put p99 improved the most. A miss-`put` at capacity used to run two full 4-row SipHash estimates. Now it reuses the incoming key's hash and pays for only one extra `key_hash` (for the victim), so the admission-contest path has much less tail cost.
- metadata-size (secondary entry): `TinyLfuCache<u64,u64>` shrank because `HashMap<K, usize, BuildHasherDefault<FxLikeHasher>>` takes no space for its hasher, while std `RandomState` stores its SipHash keys inline. Flattening `Vec<Vec<u32>>` into `Vec<u32>` doesn't change the struct size, and the sketch's heap size is unchanged by design. The headline `Node` size didn't move.

## Hurt

## Takeaway

In the parent, per-op cost came almost entirely from the hasher, not from the cache's admission or LRU logic. Hashing once per operation with a cheap, avalanche-finalised 64-bit hash, then double-hashing that into a flat sketch, removed most of the latency without changing admission behaviour: hit rate stays within noise because the sketch's geometry and counting rules are identical. The fast hasher is unkeyed, which a later variant should weigh against adversarial key sets, but that cost doesn't show up on this trace.
