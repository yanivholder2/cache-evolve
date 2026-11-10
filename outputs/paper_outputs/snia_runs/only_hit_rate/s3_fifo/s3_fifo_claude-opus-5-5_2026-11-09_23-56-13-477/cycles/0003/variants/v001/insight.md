# Variant v001 Insight

## Helped

_None measured._ The only dimension in the fitness vector is hit-rate, and it matches the parent exactly in all four trace×size cells. For a pure data-structure port, that is the intended result, not an improvement.

## Hurt

_None._ hit-rate did not move in any cell (web_0 and hm_0, at both 16384 and 131072). So the slab, `HashMap<K, Loc>` index, stamped lazy-deleted ghost and `retain` compaction changed no admission, graduation or eviction decision on real traces.

## Takeaway

This variant shows that cycle 0002/v001's occupancy-gated graduation policy can be moved off the linear `small`/`main`/`ghost` scans and the per-ghost-hit `retain` onto an O(1) index without changing its behaviour. The trace-level hit-rates match the parent bit for bit. The new `hit_miss_identical_to_linear_reference` test, which runs a verbatim copy of the old implementation as an oracle, makes that equivalence an enforced invariant. A second subtlety is also verified: the stamp check has to treat a re-ghosted key's older deque slots as stale. That check holds `ghost_live ≤ main_size` and keeps the deque at most `2 * main_size`, so the stale slots never consume ghost capacity, and that is why ghost reach stays the same as the parent's. The hypothesis's actual target was the cost dimension, and it is still unanswered. per-op-time and metadata-size were again absent from the evaluation, so this variant is a validated cost refactor whose payoff has not been measured. The next eval should measure those two dimensions on this exact variant against its parent. Because the policy is identical, any cost difference can be attributed entirely to the indexing.
