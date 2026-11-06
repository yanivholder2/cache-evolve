# Variant v003 Insight

## Helped

- metadata-size: This is the first move on this dimension since cycle 4, and the hypothesis predicted it exactly. Dropping `lfu_counter` and `timestamp` makes `Item` just `{ key, value }`, and `item_u64_u64_is_16_bytes` pins the new layout. The per-item `u32` moved into the bucket's existing `tags: Vec<u8>`, which non-lip buckets now use as a 4-byte-stride buffer. As a result the bucket gains no field: `bucket_size_matches_parent` still asserts 80 bytes, and the `AssociativeCache<u64,u64>` entry is unchanged.
- hit-rate: Not a gain, but the "bit-identical" claim held on all four cells. On lip buckets ClassicLRU gets an empty `meta` slice (`&mut []`), so its `!meta.is_empty()` rotate branch never runs. The tag mirror, BIP counter, quarter-depth insert and fused rotation are copied unchanged from the parent, and `fused_matches_reference_remove_insert` still passes against the same reference model.

## Hurt

- per-op-time: Both get and put got clearly slower at p50, which is the opposite of the predicted small win. The diff does not explain this. On the benchmarked ClassicLRU path, the only code changes are:
  - one extra fat-pointer argument on the `dyn EvictionPolicy::get_from_position` call
  - a never-taken `meta.is_empty()` check
  - `fused_evict_front_insert` and `items.remove`/`push` now move 16-byte items instead of 24-byte ones, which should make them cheaper, not slower

  A slowdown this large is unlikely to come from those changes. The likely causes are codegen/inlining fallout from the widened trait signature, now that `get_from_position` is one non-inlined virtual call carrying an extra slice argument, or run-to-run noise, which the parent insight already flagged. It is not a cost of the smaller `Item`.

## Takeaway

Moving policy-only per-item state into a policy-gated side buffer works. It removes 8 of 24 bytes per item, keeps ClassicLRU eviction order exactly, and needs no new bucket field, because the `tags` allocation's stride is chosen per policy at construction. The predicted latency benefit of smaller items did not appear. Instead there is a p50 regression on both ops that the hot-path diff cannot account for. Before keeping or discarding this change, run a same-session A/B against cycle 0007/v003. If the regression reproduces, try passing `meta` only to policies whose `meta_use() != None`, or give the lip path a `ClassicLRU` call that skips the virtual call, so the trait-signature change is tested on its own.
