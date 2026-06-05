# Cross-Cycle Insights

## Recurring wins

- **Demotion back-insertion (MRU positioning for demoted items)**: Consistently improved or maintained hit rate across cycles 0003–0006. Items with tier-advancement history benefit measurably from immediate re-promotion eligibility upon reinsertion. This is the only sustained single-lever win across the entire cohort.

- **Stash-hit filtering before acceleration**: Early cycles (0001–0002) validated that fast-tracking stash hits—items that survived stash discipline—improves hit rate. Success requires fitness gating; unconditional acceleration fails (cycle 0002, v003).

- **Deterministic positioning rules over conditional branching**: Across cycles 0003–0005, simple, unconditional MRU/LRU rules consistently outperformed occupancy-adaptive or threshold-gated insertion logic. Simplicity appears robust; conditional complexity adds noise.

## Recurring failures

- **Frequency-based eviction (LFU)**: Failed in cycle 0001 on recency-dominated workload. Access count is a weaker signal than temporal locality. Not revisited.

- **Occupancy-driven promotion without fitness checks**: Cycles 0002, 0005–0006 repeatedly show that occupancy thresholds suppress rather than improve hit rate when decoupled from item hotness or stash-hit status. Threshold alone is not a fitness signal.

- **MRU positioning for freshly promoted items**: Cycle 0004 revealed an asymmetry: back insertion benefits demoted items but destabilizes newly promoted items. Fresh promotion paths do not benefit from MRU acceleration; they require tier-level filtering first.

- **Tier-skip combined with demotion back-insertion**: Cycles 0004 and 0006 show tier-skip (stash→probing bypass) fundamentally conflicts with demotion-back-insertion discipline. Tier-skip floods the hot tier with unvetted items lacking tier-advancement proof, polluting it and degrading hit rate. The mechanisms are architecturally incompatible.

- **Intra-tier reordering and occupancy-adaptive branching**: Cycle 0004 (v002) showed that reordering within a static tier yields no hit-rate benefit. Cycle 0003 showed occupancy-adaptive destination selection adds complexity without gain. Gains couple only with tier advancement, not with position within a tier.

- **Capacity and geometric tuning (marginal lever)**: Cycle 0006 tested stash expansion and bucket reallocation. Only capacity skewing toward the probing tier yielded improvement (+0.0014), indicating capacity is a secondary lever. Stash buffering expansion failed entirely.

## Open questions

- **Is 0.684 hit rate a true local optimum or a transient plateau?** Six cycles converging on this boundary across orthogonal levers (insertion position, eligibility timing, occupancy thresholds, capacity ratios) suggests architectural constraint. Is the ceiling enforced by tier saturation, eviction cascade timing, or an asymmetry in how promotion and demotion interact? Or is headroom unlocked by untested levers (eviction ordering policies, stash depth thresholds, bucket saturation semantics)?

- **What is the mechanism of the promotion-demotion asymmetry?** Why does MRU insertion benefit demoted items (with proven tier-advancement history) but destabilize freshly promoted items? Is it eviction cadence, promotion candidacy eligibility, tier saturation, or a deeper interaction?

- **Can single-lever strategies compose at all, or is the workload saturated?** Cycles 0004–0006 suggest successful mechanisms have reached interaction limits. Do the demotion back-insertion (v003 cycle 0003) and the marginal capacity skewing (v002 cycle 0006) combine additively, or do they saturate a single underlying constraint?

- **Is tier-by-tier progression inherently superior to tier-skipping for this workload, or does the incompatibility point to a fixable architectural asymmetry?** Tier-skip failed when combined with demotion back-insertion, but succeeded briefly in isolation (cycles 0001–0002). Could a revised tier-skip policy (one that preserves probing-tier filtering or adjusts MRU semantics) unblock it?

## Per-policy notes

- **LRU-based insertion (Front)**: Adequate baseline for fresh promotions but not accelerating. Cycle 0005 confirms front insertion is not the bottleneck when demotion back-insertion is held constant; the constraint lies deeper.

- **MRU-based insertion (Back)**: Highly effective for demoted items consistently across cycles 0003–0006. Harmful or neutral for freshly promoted items (cycle 0004 asymmetry). Only drives gains when coupled with tier advancement, not with intra-tier reordering.

- **Occupancy-based thresholds and adaptive policies**: Repeatedly suppress or fail to improve hit rate (cycles 0002, 0003, 0005–0006). Without fitness gating (hotness, stash-hit status), occupancy is a poor access signal. Deterministic rules outperform conditional occupancy sensing.

- **Tier-skip policy (stash→hot-tier bypass)**: Succeeded briefly in early cycles (0001–0002) as a direct acceleration path for stash hits. Failed in later cycles (0004, 0006) due to architectural conflict with demotion back-insertion. May be incompatible with the filtered tier discipline required for high-confidence hot-tier residents.

- **Tier capacity allocation**: Marginal lever. Cycle 0006 v002 achieved +0.0014 improvement by skewing buckets toward probing (18 vs. 10), but demotion back-insertion remains the dominant mechanism. Capacity tuning alone cannot break the 0.684 ceiling.
