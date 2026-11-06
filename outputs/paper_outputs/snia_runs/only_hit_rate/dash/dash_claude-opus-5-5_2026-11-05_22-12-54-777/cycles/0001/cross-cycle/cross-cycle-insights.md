# Cross-Cycle Insights

The ledger has one cycle (0001). Each pattern below rests on that single cohort and is provisional until later cycles confirm it.

## Recurring wins

- Strengthening the protected tier (target/probing) gives the most reliable hit-rate gains. All three cycle-0001 variants attacked this weakness from a different side, and all three found headroom.
- Rebalancing the topology toward a larger normal tier and a smaller stash (v002, a 3/4 normal split) was the cleanest win. It improved hit-rate on both traces and both sizes, with no regression and no new mechanism.
- Each lever helps at a different cache size. Promotion placement helps most at the small size, topology at the large size, and the frequency-aware tail victim at both. So the levers look complementary rather than redundant.

## Recurring failures

- Topology-only changes at the small cache size do little, because the cache is too tight for the split alone to protect promoted items.
- When one lever changes alone, the other levers react. MRU promotion landing (v001) made the probing trigger fire too readily and penalised demoted items on web_0 at the large size. Expect similar interactions when changes are stacked.
- Variants have measured only hit-rate. No metadata-size or per-op-time data exists yet, so the cost side of every win is unknown.

## Open questions

- Do v002's topology, v001's MRU promotion and v003's frequency-aware victim add up when stacked, or do they compete for the same gain?
- Does the normal/stash split keep improving past 3/4 (e.g. toward 7/8)? At some point the stash should stop filtering scans; where is that point?
- What per-op-time cost do the eviction-window scan and the bucket-wide counter halving add?
- Would a stricter probing-promotion condition, or a softer demotion position, remove the MRU-promotion regression on web_0 at the large size?
- Should stash hits count toward the frequency filter?

## Per-policy notes

- Baseline two-tier policy (normal/stash, target/probing): its protected tier barely protects anything. Promoted items are either evicted quickly when they land at the front, or crowded out of a small normal tier.
- v002 (topology 3/4 normal): current Winner and the recommended parent for stacking.
- v001 (MRU promotion/demotion placement): strong at the small size. It needs a gated probing trigger before it is safe at the large size.
- v003 (frequency-aware LRU-tail victim): gains at both sizes, larger on hm_0. Its per-op cost has not been measured.
- Traces: hm_0 responds to changes more readily than web_0. web_0 is where interaction regressions have appeared.
