# Variant v002 Insight

## Helped

- per-op-time (get p50): `get` p50 dropped clearly below v001, so the hypothesis is **confirmed** on its stated criterion. Gating the `s1` probe in `promotion_get` on `borrowed[s1] != 0` skips the 2-slot `s1` scan on most misses. This supports the rationale: in steady state most stash buckets hold only their own native keys, so the one-byte counter is usually zero. Loading and branching on it costs less than the scan it replaces.
- hit-rate (held, as required): Hit rate is identical to v001. This is the hypothesis's bookkeeping check: the counter stays exact across `put_stash_at_index`, `remove_stash_at` and `put_at(Slot::Stash)` evictions. So the gate never skips an `s1` bucket that holds a borrowed key, and lookups and mutations behave exactly as before.

## Hurt

- per-op-time (put p50 and especially put p99): `put` got slower, and its tail grew much more than its median. All the bookkeeping cost falls on the write path. `put_stash_at_index` hashes the incoming key again to get `s0`, even though `stash_insert` has just computed it. Every stash eviction, in both `put_stash_at_index` and `put_at(Slot::Stash)`, hashes the evicted key again in `note_stash_removal` to decide whether to decrement. The tail grew most because the put paths that evict from a full stash pay for two extra hashes plus a read-modify-write on `borrowed`.
- per-op-time (get p99, weak): `get` p99 rose slightly. On rare `s1` hits, `stash_hit` now promotes through `remove_stash_at`, which adds one more hash of the removed key and a counter update to the slowest get path. The change is small and may be noise.

(metadata-size did not change: `borrowed` lives in `SegmentCore`, one `u8` per stash bucket. `Item` and `Dash` are the same size as before.)

## Takeaway

An exact per-stash-bucket count of borrowed items is a sound, lossless filter for the `s1` miss probe. It recovers most of v001's get-p50 gap without touching hit rate or per-item metadata. Its cost lands on `put`: the counter is kept by rehashing keys that the insert/evict path already hashed or could classify cheaply. Next, keep the gate but remove the redundant hashing. Pass `s0` (or a borrowed/native flag) from `stash_insert` into `put_stash_at_index`, and tell borrowed from native evictees without calling `get_index` again. That should bring put p99 back while keeping the get win.
