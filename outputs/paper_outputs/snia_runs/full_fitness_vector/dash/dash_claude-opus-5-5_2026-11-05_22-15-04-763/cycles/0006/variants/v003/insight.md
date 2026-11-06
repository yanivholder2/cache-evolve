# Variant v003 Insight

## Helped

- per-op-time (`get` p99): The `get` tail improved, as the hypothesis predicted. Before, the forced third-hit promotion of a deferred item ran `remove_at` → `promote_to_target` → `demote_from_normal_bucket`. Now it runs a single `swap_promote`. That removes one stash-bucket shift and the separate length decrement and increment. The third-hit promotion was the slow `get` path that v002 moved there, so the tail is where the saving shows. The `get` median did not move, which fits: the fused path only runs on that rare case.

## Hurt

- per-op-time (`put` p99, and slightly `put` p50): The `put` tail regressed noticeably, which the hypothesis did not expect. The hypothesis treats the change as cost-only, and the hit-rate configurations match bit-for-bit, so this is not a policy effect. It is a cost or code-layout effect on the write path. One likely cause is a second call site for the fused helper: the non-deferred second-hit promotion. Inlining that helper, or the larger `get`/promotion code, may have changed how the shared bucket-insert and demotion code used by `put` gets inlined or laid out. Another possible cause is that `swap_promote` now runs on a path `put` reaches through stash hits. This is inferred from the hypothesis and the fitness vectors only; the overlay source was not readable from this session, so neither cause was confirmed in the code.

## Takeaway

Fusing the stash-promotion chain into one in-place rotate does cut the `get` tail. That confirms part of v002's remaining `get` p99 comes from moving data within buckets. The policy is identical (hit rate and metadata size did not change). The saving is small, though, and `put` gave back more tail latency than `get` gained. So shift-merging alone is not a net win. Before building on it, check whether the `put` regression comes from the second (non-deferred) call site or from inlining and code layout. A good test is to limit `swap_promote` to the third-hit path only, or to mark it `#[inline(never)]`.
