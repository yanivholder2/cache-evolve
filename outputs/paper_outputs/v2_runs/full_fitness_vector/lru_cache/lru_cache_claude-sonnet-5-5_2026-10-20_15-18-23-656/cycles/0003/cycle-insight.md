# Cycle 0003 Insight

## Cohort outcome

Three variants refined the segmented LRU: u32 links on the fast segmented cache (v001), segmentation built onto the u32-link plain LRU with the segment bit packed into `next` (v002), and lazy demotion moved from `get` to `put` (v003). All three were Winners, but each paid for it in per-op-time. Nothing improved speed, and the only gains were metadata size (v001) and hit-rate (v002).

## Patterns

- Every variant slowed per-op-time, including tails. This contrasts with cycle 0002, where u32 links and the hasher were pure wins on the plain LRU. Once the policy is segmented, link-width and bit-packing changes no longer bring speed.
- Narrowing links shrank the cache (v001), but the added widening and NIL checks, plus the hash fold, cost latency. Locality did not pay off at this workload size. The saving is size only, and it is not confirmed that u32 links on a segmented policy are a speed win.
- v002 again showed that the cost of segmentation is the promotion/demotion relinking on the hot path. Packing the flag keeps per-node size flat but cannot remove that work. The container grows from the extra head/tail pair, and a per-node metric hides this.
- v003 showed that deferring demotion only moves the relink work. It landed on `put`, the more latency-sensitive path, and `get` did not improve. Hit-rate was preserved.
- The remaining time cost of segmentation is the promotion/demotion pointer surgery itself, and reshuffling it between operations does not fix that.
- Variant authors could not read the diffs. Attribution among the hash fold, index conversion and flag masking is inferred.

## Open questions

- Which piece of v001 caused the slowdown: the hash fold or the u32 conversions? The fold was bundled with the layout change.
- Can promotion be made cheaper without eliminating it, for example by promoting only when a node is far from the head, or by promoting on a sampled or deferred basis?
- Can the segment be stored implicitly, such as by list membership, so that no masking is needed on link access?
- Does segmentation's hit-rate gain justify its latency cost on workloads other than the scan-heavy trace?

## Continuation

Stop shifting demotion work between `get` and `put`, and stop bundling the hash fold with layout changes. Next, run an isolated test of the fold against the u32 conversion cost. Then attack promotion cost directly: skip the relink for nodes already near the head, or batch or amortize promotion. Treat the cycle-0002 merged candidate (segmented, u32 links, folded Fx, move-based eviction) as the baseline, and judge new variants by whether they recover speed without losing hit-rate.
