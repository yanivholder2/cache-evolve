# Variant v003 Insight

## Helped

- metadata-size: The change dropped the duplicate key from `Entry`, which now holds only `value`, `freq` and a zero-sized `PhantomData<K>`, so the per-entry record shrank exactly as H03 predicted. Removing the `slots` and `free` `Vec` headers also shrank the `S3Fifo` struct itself.
- per-op-time (`put`, tail): The expected `put` regression did not show up. The miss/evict path lost the slab bookkeeping: there is no more `free.pop`/`free.push`, no `slots.push(None)` growth, and no `Option::take` on release. That appears to offset the extra hash probe that `evict_main` and `evict_small_inner` now make per CLOCK step. `put` p50 held and the tail improved. This variant cannot tell whether the tail gain is real or noise.

## Hurt

- per-op-time (`get`, the summary): H03 claimed that a "single-probe" `get` would be faster, and the measurement says the opposite. `get` lost the dependent `slots[i]` load and the `Option` branch, but each hashbrown bucket now stores `(K, Entry<K,V>)`, which is 24 bytes (up from the parent's 16-byte `(K, u32)`). That means wider probe strides, fewer buckets per cache line, and some buckets that straddle two cache lines. The probe is the dominant cost of `get`, so making it touch more memory cost more than the separate slab load ever did. Both p50 and p99 moved in the same direction, which makes noise a weaker explanation than it was for the parent's unchanged `get`.

(hit-rate did not move. An identical hit-rate confirms that the refactor is policy-neutral: eviction order, ghost ring, `fp`, adaptive split and the `freq > 1` graduation are unchanged.)

## Takeaway

On this workload, putting the payload into the hash map to save one indirection made `get` slower, because the cost of `get` comes from the hash probe, not from the slab hop. Making each bucket wider (16 → 24 bytes, not a power of two) hurt the probe more than removing the dependent load helped. Counting dependent loads is not enough to predict `get` latency: bucket width and cache-line layout of the index matter more. A slimmer `Entry` helps metadata-size, but not if it is placed inline in the probe table.
