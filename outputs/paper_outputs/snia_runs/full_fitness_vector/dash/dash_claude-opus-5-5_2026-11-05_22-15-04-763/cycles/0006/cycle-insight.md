# Cycle 0006 Insight

## Cohort outcome

All three variants followed up on cycle 0005's open questions. v001 allowed the deferral only when the stash bucket has free slots (parent 0005 v002). v002 skipped the MRU reorder on near-MRU normal-tier hits (parent 0004 v002). v003 fused the stash-promotion chain into one in-place rotate (parent 0005 v002). There are no Winners. Each variant moved the metric it aimed at only slightly, or not at all, and each paid for it on a different axis.

## Patterns

- A gate based on how full the stash bucket is does nothing in practice. At steady state the stash is almost always full, so v001 hardly ever deferred and behaved much like 0004 v002 again. It lost the `put` p99 gain, and the hit-rate change only moved from the large cache to the small cache. Fill level cannot tell a harmful deferral from a harmless one.
- The MRU shift is not where the `get` p50 gap comes from. v002 kept hit rate bit-identical and left p50 flat. Scan order, scan direction, cascades and now the MRU shift have all been ruled out, which leaves the stash probe on every normal-tier hit as the last suspect on the common hit path.
- Moving data within buckets accounts for part of the `get` tail. v003's fused rotate kept the same policy and made `get` p99 better. The gain is small, though, and `put` p99 got worse by more. The policy is identical, so the regression comes from cost or code layout, probably from the second call site or from inlining.
- Both cost-only variants (v002, v003) made a p99 worse on a path they were not expected to touch. This matches cycle 0005's warning about noise. It may also come from codegen or inlining side effects. Small tail changes need a rerun before anyone draws conclusions from them.

## Open questions

- If the stash probe holds the `get` p50 gap, how can it be made cheaper without reordering the lookup? Cycle 0003 v001 already checked stash last, and `get` got worse because stash hits make up a large share of all hits. Possible routes: a cheaper stash membership check, or skipping the stash probe for keys known not to be there.
- If `swap_promote` is limited to the third-hit path or marked non-inline, does its `get` p99 gain stay without the `put` regression?
- Is there a better signal for the deferral than fill level, for example whether the stash slot that would be evicted holds an item still waiting for its second hit?

## Continuation

Stay on cycle 0005 v002, or on 0004 v002 for p50 work. Drop the fill-level stash gate and the near-MRU reorder skip. For `get` p50, go after the cost of the stash probe itself. Do not move stash to the end of the lookup again, since cycle 0003 showed that is worse. For `get` p99, carry the fused rotate only if it is kept to the third-hit path or isolated from inlining, and rerun any result that rests on a small tail change.
