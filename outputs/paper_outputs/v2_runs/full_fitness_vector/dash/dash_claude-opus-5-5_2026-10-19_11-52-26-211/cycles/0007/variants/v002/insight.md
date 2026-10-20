# Variant v002 Insight

## Helped

- per-op-time (get p50): `get` p50 went below the parent's level, so the hypothesis is **confirmed** on its stated criterion. `promotion_get` now probes the `probing_idx` bucket only when `foreign[probing_idx] != 0`. That supports the rationale: hottest→probing promotions are rare next to target fills, so most probing buckets hold only native keys. A one-byte load plus a branch is cheaper than scanning the bucket on a miss.
- per-op-time (get p99): the `get` tail also fell. The gate removes a whole bucket scan from the slowest miss path (`s0` → target → probing → `s1`). It does this on top of the existing `borrowed[s1]` gate, so a full miss can now skip both neighbour probes.
- hit-rate (held, as required): hit rate is identical to the parent. This is the hypothesis's bookkeeping check. It shows that `foreign` stays exact: `put_at(Slot::Probing)` increments it, and `note_normal_removal` decrements it using the `true_target` the demotion path already computes. So the gate never skips a probing bucket that holds a resident key.
- per-op-time (put p99, likely not caused by this change): `put` p99 fell sharply, back down near `put` p50. The new code cannot explain this. The write path only gained work (one increment on probing inserts and one compare-and-decrement per normal evictee), and the stash rehashing in `note_stash_removal` / `put_stash_at_index` that the parent blamed for its put tail is still there. Treat this as run-to-run variance until a re-measurement reproduces it.

## Hurt

(No dimension got worse. `put` p50 is flat, which matches the claim that counting foreign items without rehashing costs nearly nothing on the write path. metadata-size did not change: `foreign` is one `u8` per normal bucket in `SegmentCore`, and `Item` and `Dash` are the same size as before.)

## Takeaway

The exact-counter gate from the parent, which skipped stash probes, also works for the normal tier's probing bucket. It works better here: the slot type already says whether an inserted item is foreign, and the demotion path already knows the evictee's `true_target`. So the counter needs no extra hashing and costs nothing measurable on `put`. Generally, an exact occupancy counter is close to free when every mutation that changes it already has the classifying information on hand. The stash `borrowed` counter still rehashes in `note_stash_removal` and `put_stash_at_index`. That is the remaining cost to remove, and the large `put` p99 drop needs a re-measurement before anyone counts on it.
