# Variant v002 Insight

## Helped

- per-op-time: Swapping the key map's default SipHash for a zero-state FxHash-style `FastBuildHasher` removes the largest per-hit cost on the read path, so both the `get` and `put` medians and tails dropped; the `put` path benefits because its `map.get` and `map.insert` also go through the hasher.
- metadata-size: The `LruCache` struct shrank because `FastBuildHasher` is a zero-sized type, so the map no longer carries the 16-byte `RandomState` seed; the `Node` summary is unchanged because no node field was touched.

## Hurt

## Takeaway

The change is policy-neutral by construction: `select_victim`, `demote_prot_head`, `unlink`, and `push_back` are unchanged, and the hit-rate did not move, so the gain is entirely in the map's hashing cost. The hasher folds each word with a rotate, xor, and multiply and returns the raw state from `finish()`, which is fast but leaves the bucket-index bits as the least-mixed part of the output, so the gain depends on the workload's keys spreading well under this mix. The `write` byte path zero-pads the tail and is only exercised by non-integer keys such as `String`, so the speedup on integer-keyed workloads is the one this variant actually demonstrates.
