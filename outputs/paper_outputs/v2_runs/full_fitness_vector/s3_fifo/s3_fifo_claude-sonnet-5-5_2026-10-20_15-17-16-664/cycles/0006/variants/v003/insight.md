# Variant v003 Insight

## Helped

- per-op-time: Both get and put latencies moved down only slightly. This fits the hypothesis: skipping the saturated-frequency store, moving entries by owned `pop_front`/`push_back`, and pre-reserving the deques removes a little work and some growth reallocation. The shifts are small enough to be within run-to-run noise, so the credit is tentative. I could not read the overlay source in this session, so I did not check whether the code matches the hypothesis.

## Hurt

- None. Hit-rate and metadata-size did not move, as the hypothesis required. Eviction order, ratios and the `Entry` layout were left unchanged.

## Takeaway

This was a behavior-preserving micro-optimization. The hit-rate and layout invariants held, but the gain was marginal. At a 5% small queue on this trace, the saturated-frequency store, the small-to-main move and deque growth were not the main costs. The put tail is likely dominated by something else, such as ghost-queue lookups or the eviction scan, so the next change should target that instead of more allocation or store trimming.
