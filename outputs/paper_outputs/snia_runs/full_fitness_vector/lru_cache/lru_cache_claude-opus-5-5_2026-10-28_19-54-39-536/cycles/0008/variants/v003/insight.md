# Variant v003 Insight

## Helped

- hit-rate (hm_0 @ 131K): Shrinking protected from 80% to about 2/3 helped hm_0, against the hypothesis's expectation. The extra probation time let more of hm_0's second-touch keys get their promoting hit before being evicted. The smaller protected segment did not expose its loop-resident working set to eviction. So at this size hm_0 was limited by its probation window, not by protected space.

## Hurt

- hit-rate (web_0 @ 131K): web_0 lost hits, the opposite of what the rationale predicted. A wider probation window helps every key, which is not the same as a ghost that helps only returning keys. Moving about 13% of capacity out of protected evicted web_0's reused keys sooner than the longer probation stay rescued new ones. In net, web_0's 131K hits come from protected-segment depth, so the ghost result does not carry over to a static split.
- hit-rate (summary): The web_0 loss is slightly larger than the hm_0 gain, so the change is roughly neutral on the summary and comes out slightly negative. The 16K points are unchanged, which confirms the `LARGE_CAPACITY` gate kept small sizes identical to the parent.
- per-op-time (get p50, put p50/p99): These rose even though `get`, `on_hit`, `unlink`, `push_back` and `put` are unchanged from the parent. The diff touches only `try_new` and the tests, and get p99 actually fell. Most likely this is run-to-run benchmark variance, not a code cost. One alternative remains: if the benchmark uses a capacity above 32768, the new split changes how often `on_hit` takes the promote-plus-demote branch. A re-run would tell these apart.

## Takeaway

At 131K, moving the probation/protected split from 80/20 to about 67/33 trades hits between the two traces: hm_0 gains and web_0 loses by slightly more. So a static split is not a free substitute for ghost-guided extra probation time on web_0. The benefit the ghost gave web_0 depends on targeting returning keys, and a wider window for all keys dilutes it. Also, because the hot paths are identical, the per-op-time shift here gives a rough idea of the benchmark's noise floor. Future comparisons of per-op-time changes between variants should be read against that noise.
