# Variant v003 Insight

## Helped

(No measured dimension improved. The goal of this change was to lower `metadata-size`, but this run's fitness vector has only `hit-rate`. Neither this variant nor its parent was scored on `metadata-size`, so the main claim of H03 is still untested, not confirmed. On paper the saving is real. Each of the 332 ghost slots now costs one `u64` in a preallocated ring plus one `u64 -> u32` count entry. The parent kept two owned `K` copies per slot, one in `ghost_q` and one in `ghost_set`. The `node.key.clone()` on every `small` eviction is also gone, because the fingerprint is computed from a borrow before `map.remove`.)

## Hurt

(No measured dimension regressed. `hit-rate` is unchanged from the parent. That is the outcome the hypothesis predicted for a representation-only port, and it is the refutation condition H03 set itself. `small_target`, `ghost_cap`, `MAX_FREQ`, `get()` and the `main` branch match the parent exactly.)

## Takeaway

On workload-a at size 2048, the policy does not depend on how the ghost is represented. Holding 64-bit fingerprints instead of keys reproduced v003's admission decisions closely enough that `hit-rate` did not move. This held even though the count-based ring also changes one edge case. In the parent, popping a stale `ghost_q` entry could evict a newer ghost membership for the same key from `ghost_set`. In this variant, the per-fingerprint occurrence counts keep that membership alive. That quirk is evidently too rare on this trace to matter. The fingerprint ring is therefore a safe, policy-neutral base for later S3-FIFO variants. Whether it actually wins on `metadata-size` can only be settled once that dimension is included in the evaluation.
