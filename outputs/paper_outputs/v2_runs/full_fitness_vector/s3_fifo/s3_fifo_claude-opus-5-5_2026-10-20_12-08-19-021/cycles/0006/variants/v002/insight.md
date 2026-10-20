# Variant v002 Insight

## Helped

- metadata-size: The `key` field moved out of `Entry` and into a parallel `keys: Vec<Option<K>>`, so `Entry<u64,u64>` is now `{ value, freq }`. The `entry_is_16_bytes` test confirms the 16-byte size. There is a catch. `Option<u64>` is itself 16 bytes, and the parent's `Option<Entry>` had no niche, so it was 32 bytes per slot. That means the real slab footprint per slot is about the same as before (16 + 16 vs 32). Only the hot array got smaller. The `S3Fifo` header also grew by one `Vec` header for `keys`.
- per-op-time (`get`, the summary): `get` still does one `index` probe and one dependent load, as the hypothesis required. Two things changed on that path. The `Option` discriminant match is gone, and the load now hits a `slots` array with entries half as wide as the parent's 32-byte `Option<Entry>` slots. `get` never touches `keys`, so more hot entries fit per cache line. The large drop at p99 is probably also partly a return from the parent's tail, which the v001 insight already flagged as unexplained by its code and likely noise.
- hit-rate: This did not move. That is the expected result and confirms the refactor is policy-neutral: eviction order, graduation, the `capacity/64` floor and the ghost ring all behave exactly as before. `release` returning `K` to `ghost_push` keeps the same fingerprint flow.

## Hurt

(No dimension regressed. `put` p50 is flat. On a miss, `put` now writes to two arrays (`keys[i]` and `slots[i]`) instead of one, but that cost does not show up, and `put`'s tail got slightly better rather than worse.)

## Takeaway

The win here comes from splitting cold data from hot data, not from saving memory. The key is needed only on release and ghost push, so moving it out of the array that `get` reads speeds up `get` while keeping the narrow `(K, u32)` index bucket, which cycle 0005 v003 lost. The `metadata-size` metric measures only `size_of::<Entry>`, so it overstates the saving: `Option<K>` in `keys` gives back the bytes. The next layout lever is to make `keys` niche-free (for example, a plain `Vec<K>` plus liveness tracked through `free`) so the total per-slot footprint actually shrinks. One more point: released slots now keep their stale `V` until they are reused, which delays `Drop` for non-trivial value types.
