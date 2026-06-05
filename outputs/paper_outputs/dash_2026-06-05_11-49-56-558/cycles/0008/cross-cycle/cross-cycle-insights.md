## Recurring wins

- **Hot-tier topology expansion** has been the single highest-leverage lever across cycles 1–3 and 8. Each increase in the normal/stash split (equal halves → 75/25 → 7/8) produced positive gains; the working set consistently fits more comfortably as hot-tier capacity grows. Returns are now clearly diminishing but not yet zero.
- **MRU insertion position** is universally beneficial on workload-a. Every cycle that experimented with non-MRU placement for newly arrived, re-admitted, or re-promoted items regressed. Recency position encodes a stronger future-access signal than any alternative tested.
- **Eviction selectivity via zero-access frequency guard** (protecting items with ≥1 in-tier hit from eviction by never-accessed newcomers) produced the largest single-cycle gain in cycles 6–8. The zero-access cutoff is empirically correct for this workload; raising it hurts.
- **Unconditional second-chance stash recovery** (re-inserting stash-evicted items at the LRU end of their target bucket) stacks cleanly with the frequency guard because the two mechanisms target distinct pipeline stages — probing eviction selectivity vs. stash overflow recovery.
- **Stacking orthogonal mechanisms** consistently pays off. Topology + MRU re-entry, frequency guard + second-chance recovery, and topology + promotion gating all compounded without destructive interference when the mechanisms addressed different loss sites.

## Recurring failures

- **LFU / frequency-based eviction in any form** has never beaten recency on workload-a. Per-hit co-resident aging, threshold-gated decay, and soft-decay variants all regressed or tied. The access distribution punishes frequency discrimination; workload-a is recency-dominant.
- **Ghost set re-admission** has been noise-level across all position and size configurations tested in cycles 4–5. The stash boundary is not a meaningful hit-loss site on this workload; conditional event-driven re-admission provides no net benefit.
- **Withholding MRU for probationary filtering** (LRU entry for newly inserted, re-admitted, or demoted items) consistently hurt. Items displaced from prime position are re-accessed before eviction on workload-a, so deferral delays useful promotion without filtering cold noise.
- **Weakening or bypassing stash traversal** (front-door cold bypass, direct-insert modes) always regressed. The stash functions as a load-bearing admission filter; removing it allows cold items to displace warm residents at a net cost.
- **MRU-end insertion for second-chance items** regressed sharply; it grants unproven items immediate promotion candidacy and displaces earned residents. LRU-end second-chance insertion is definitively correct and should not be revisited.
- **Timestamp-based promotion** interacted destructively with the frequency-guard baseline because creation-time contamination allows cold-but-recently-inserted items to outcompete hot-but-older ones. The net interaction is harmful, not independent.

## Open questions

- Is there an inflection point in the normal/stash split (beyond 7/8) where stash becomes too shallow to buffer burst arrivals and the marginal loss from reduced stash headroom overtakes the geometry gain?
- Would soft-capping `lfu_counter` to prevent counter staleness yield any improvement once geometry ceases to help? This is the clearest untested policy axis remaining.
- Could geometry (7/8 split) and the stricter two-hit promotion gate stack for further gain, given they target distinct sites (capacity vs. promotion selectivity)?
- Would timestamp-based promotion recover if creation-time contamination were eliminated — e.g., by only recording a timestamp on the first *hit* rather than on insertion?
- Do frequency-based or admission-control schemes unlock real gains on a workload with heavy scan traffic or bursty cold-item arrivals, where these mechanisms are structurally better matched?

## Per-policy notes

- **ClassicLRU / recency (MRU insertion):** The dominant policy class for workload-a without exception. Recency signal is reliable across the entire access distribution; every departure from it has cost hits.
- **LFU / frequency eviction:** Structurally incompatible with workload-a in all forms tested — per-hit aging, threshold-gated decay, and access-count-gated variants all lost. Should remain retired unless a different workload trace motivates revisiting.
- **Zero-access frequency guard:** High-value selective eviction mechanism; confirmed correct at the zero-access threshold. Single-access protection is the right boundary — raising the cutoff over-shrinks the protected set. Counter resets on stash demotion have not been confirmed harmful.
- **Second-chance stash recovery:** Additive with the frequency guard; unconditional is better than ghost-set conditional; LRU-end insertion is mandatory — MRU-end insertion regressed clearly and is definitively contraindicated.
- **Ghost set:** Retired. Noise-level across multiple configurations in cycles 4–5. The conditional re-admission mechanism cannot compensate for the stash boundary not being a meaningful loss site.
- **Timestamp-based LRU promotion:** Shows some standalone benefit under Zipfian distribution but is not compatible with the frequency-guard baseline due to creation-time contamination. Not a drop-in supplement; would need a hit-time-only timestamp variant to be viable.
- **Segmented-LRU asymmetry (promotions at MRU, demotions at LRU):** Contraindicated on workload-a; demoted items retain residual heat and losing MRU position in the destination tier accelerates churn rather than improving selectivity.
- **Stash-admission gating (LFU-gated promotion, bypass policies):** Both forms regressed. The stash's role as an admission filter is load-bearing; filtering at the stash boundary removes a productive quality gate.
