# Variant v002 Insight

(Note: the overlay source could not be listed in this session, so this is based on the hypothesis, the fitness vectors and the parent's insight. It is not based on a code diff.)

## Helped

- metadata-size: No help and no harm. The hypothesis kept the 40-byte item layout, and the bit-packed second-hit flag is unchanged.

## Hurt

- per-op-time: Both get and put latency got worse than the parent, at p50 and at p99. The early-return shortcut for items already at MRU in their target bucket, and the "write the hit bit only if clear" guard, were meant to remove work. They add a check on every hit, and the check is unpredictable when it only sometimes succeeds. A conditional load-and-branch can cost more than the redundant store it avoids. Put latency also rose even though the change targets the get-hit path. That suggests run-to-run noise or layout and codegen effects, not just the logic change.
- hit-rate: Unchanged. This is consistent with admission semantics being untouched, so the shortcut does not alter the eviction order. It gave no benefit here.

## Takeaway

Skipping "already hot" promotions is not a latency win in this design. The branch guards cost at least as much as the remove-and-reinsert and store they avoid, and the hit path was already cheap. The hypothesis's premise, that redundant promotion work dominated get latency, was not borne out. Future latency work should look elsewhere, such as the lookup or locking path, and should re-measure to separate real regression from noise.
