# Variant v001 Insight

## Helped

- metadata-size: Moving `freq: u8` out of `Entry` into a parallel `freqs: Vec<u8>` leaves `Entry` as `{ value, PhantomData }`. The 7 padding bytes that alignment added after the 1-byte `freq` are gone, and the `entry_is_8_bytes` test confirms the dense `[V]` slab. This is a real per-slot saving, unlike v002's key move, which `Option<K>` in `keys` mostly offset. Per slot, the true footprint drops by about 7 bytes (8 + 1 + 16 for `keys`, versus 16 + 16). The `S3Fifo` header grows by exactly one `Vec` header for `freqs`.
- hit-rate: Unchanged, matching the parent bit for bit. The refactor is policy-neutral. `freqs[i]` is reset to 0 on both alloc paths, so `free.pop()` reuse does not leak a stale frequency. `evict_main`, the `freq > 1` graduation test and `MAX_FREQ` read the same values they did before, just from a different array.

## Hurt

- per-op-time (`get`, the summary): `get` now does two bounds-checked accesses, a read-modify-write on `freqs[i]` and then a read of `slots[i].value`. The parent did one, because freq and value shared a 16-byte entry and therefore a cache line. The hypothesis bet that these two independent loads would overlap and cost less than the wider stride. In practice, the extra access plus its bounds check put a second cache line on every hit, and `get` slowed down instead of speeding up. At capacity 2048, the parent's 32 KB slab already fit in L1/L2, so halving the stride had little locality left to gain.
- per-op-time (`put`): The code change on `put` is tiny: one extra `freqs.push(0)` or `freqs[i] = 0` per alloc, and `freqs` access during eviction scans. Yet `put` regressed in proportion to `get`. That much `put` slowdown is hard to explain from the diff. Some of the shift across both ops is probably benchmark-environment drift rather than the layout change, and re-measuring parent and child on the same run would separate the two.

## Takeaway

Splitting out a field that `get` *does* touch is a different trade from v002's split of the key, which `get` never reads. Moving the hot `freq` byte away from `value` turns one co-located cache line into two accesses on every hit. That gives up `get` latency for a smaller `size_of::<Entry>`, and at this cache size the latency loss outweighs the small locality gain. If the metadata saving is worth keeping, the better path is probably to pack `freq` into spare bits beside the value or key, so the hot data stays in one line, rather than a separate array. Note: the requested `h01_split_report` settled `small_size` figure was not among the eval artifacts I could find. Because the policy code is unchanged and hit-rate matches the parent, it should equal the parent's settled split, but I did not verify that.
