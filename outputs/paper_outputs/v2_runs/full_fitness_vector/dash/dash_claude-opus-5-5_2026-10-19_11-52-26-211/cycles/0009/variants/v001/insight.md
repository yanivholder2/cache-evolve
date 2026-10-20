# Variant v001 Insight

> Note: I could not list the overlay source files in this session, so I couldn't read the diff directly. This interpretation is based on the change H01 specifies and on the two fitness vectors.

## Helped

- per-op-time (get p50): `get` p50 fell below the 69 ns threshold, so H01 is **confirmed** on its main criterion. With one flat `Vec<Item>` per segment tier, bucket `i`'s slots are at `i*N`, so a probe skips the bucket-header → heap-buffer hop. A hit's recency move is a direct `#[inline]` `touch` on a slice, not a vtable call into `Box<dyn EvictionPolicy>`. Cycle 8 found that removing hashes or the `Box<dyn Bucket>` hop alone did nothing. This result points to the remaining dependent load plus dynamic dispatch as part of the median floor.
- hit-rate (held, as required): hit rate exactly matches the parent. The slice helpers (`find_in`, `touch`, `insert_at`, `remove_at`, `victim`) therefore reproduce the old policy's ordering, Front/Back placement and victim choice. The `borrowed`/`foreign` gates were still accurate after the rewrite of `promotion_get`, demotion and the stash paths.
- metadata-size (held): `Item` and `Dash` stayed the same size, because the flat arrays and per-bucket `u8` lengths live behind `SegmentCore`'s heap allocation.

## Hurt

- per-op-time (put p50): the median `put` got slower. In a flat fixed-capacity slice, `insert_at(Front)` and `remove_at` must shift the slots that follow in place. The parent's per-bucket policy could reorder more cheaply. Demotion and stash moves now pay these shifts on two slices per move.
- per-op-time (put p99): the `put` tail worsened the most of any metric. Multi-step paths like `promote_to_target` → `demote_from_normal_bucket` → `stash_insert_known` now chain several memmove-style shifts. In the parent this tail was already suspect: its insight treated the low p99 as possible variance, so part of this rise may be regression to the earlier level, not a real cost.
- per-op-time (get p99): the `get` tail rose a little even though the median fell. A plausible cause is the move-on-hit in `touch`, which now shifts items across the bucket slice on each hit, plus promotion work on the slow path. The rise is small enough that it may be noise.

## Takeaway

Flattening bucket storage into per-tier contiguous arrays and dispatching the policy statically did lower the median `get`. So indirection and the vtable call were part of the floor, not slot walking alone. The cost moved to mutation: position-ordered recency in a fixed slice turns each reorder, insert and removal into in-place shifting, and `put` got worse at both p50 and p99. Next, keep the flat layout but replace the position-ordered recency with something that doesn't move items, such as per-slot age/clock bytes or a small index permutation. That should keep the `get` gain without the shift cost on `put`.
