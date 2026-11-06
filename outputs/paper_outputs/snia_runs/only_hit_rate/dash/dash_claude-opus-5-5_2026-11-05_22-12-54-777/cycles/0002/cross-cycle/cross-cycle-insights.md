# Cross-Cycle Insights

## Recurring wins

- Strengthening the protected (normal/target) tier keeps paying off. Both Winners so far address the baseline's weak protection of promoted items, one through topology and one through eviction choice.
- Making single, isolated changes on top of the previous Winner has produced clean, attributable gains. Both Winners were one-lever changes with no regression in any cell.
- The 24 normal / 8 stash (about 3/4) split is a stable foundation. Later changes built on it without having to retune it.
- Changes that act inside a bucket (choosing the frequency-aware victim at the LRU tail, deciding where a promoted item lands) are what move the small cache size. Topology changes mostly move the large size.
- The frequency-aware tail victim held up across both cycles. It helped at both sizes on its own, and it added cleanly on top of the topology change.

## Recurring failures

- Topology-only changes at the small cache size: two cycles in a row they did little there, because the cache is too tight for the split alone to help.
- Pushing the normal/stash ratio past 3/4. With admit-everything-to-stash, the stash is the only way into the protected tier, so shrinking it starves promotion on hm_0 before scan filtering on web_0 gets worse. The ratio has peaked.
- MRU-end promotion, together with the trigger and demotion changes that came with it, regressed web_0 in both cycles (at the large size in cycle 1, a small cost in cycle 2). It helps hm_0 but does not generalise across traces.
- Bundling several levers into one variant (cycle-2 v001) made the regressions impossible to attribute.
- Cost has never been measured. Per-op-time and metadata-size have gone unmeasured for two cycles, even though the Winners add work to the eviction path.

## Open questions

- What do the eviction-window scan and the bucket-wide counter halving cost in per-op-time? Would a narrower window or a lower cap keep the hit-rate gain?
- Does MRU promotion alone still add anything on top of the frequency filter, or does the filter already capture the small-size gain on hm_0?
- Which piece of the MRU bundle causes the web_0 loss: mid-stash demotion or the hit-bit probing guard?
- Could admission or stash-promotion changes (a longer stash wait, promotion on first re-reference) serve hm_0's long-gap reuse without growing the stash?

## Per-policy notes

- Baseline (normal/stash with target/probing tiers): the protected tier gives almost no protection by default. Promoted items are either evicted quickly when they land at the Front, or crowded out when the normal tier is small.
- Topology variants: they help most at the large size and have peaked at about a 3/4 normal share. Treat the split as settled.
- Frequency-aware LRU-tail victim (with stash hits counted): the current lineage Winner. It improved all trace/size cells and gained most at the small size. Its eviction-path cost has not been measured.
- MRU-end promotion variants: they change how readily the probing trigger fires and how demoted items are treated, and they show interaction effects with other levers. They are consistently good on hm_0 and fragile on web_0.
- Traces: hm_0 is the most responsive trace in both directions, giving the largest gains and the largest regressions. web_0 is the one that catches over-aggressive promotion.
