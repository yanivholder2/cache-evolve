# Cycle 0004 Insight

## Cohort outcome

All three variants extended the cycle-3 frequency-demotion baseline in different directions — scan-fraction widening, ghost-window expansion, and frequency-aware eviction in probationary. Only the probationary extension (v003) won, by ensuring that the freq counter preserved across demotion actively shields recently-warm entries at both eviction sites.

## Patterns

- The frequency counter and counter-halving-on-demotion design have been the load-bearing mechanism since cycle 3, but it was applied asymmetrically: protected-segment demotion respected frequency, while probationary eviction discarded it via blind LRU. v003 closed that gap — the signal survived the segment boundary but was being thrown away at the next eviction decision.
- Scan-fraction tuning above 1/4 in the protected segment is now confirmed to offer negligible returns on this workload. Going from 1/4 to 1/3 moved hit rate only marginally, indicating the frequency variation near the LRU end of protected is mostly captured by the narrower window already.
- Ghost-window expansion to full capacity regressed hit rate: reuse events at distances between probationary capacity and total capacity on this trace are not reliable locality signals, and fast-pathing those keys into protected pollutes it. The effective ceiling for the ghost window sits well below total capacity, and the ghost-window expansion thread is now largely exhausted.

## Open questions

- Is the probationary LRU-half scan fraction robust, or should it be adaptive? A workload with longer reuse distances or a more skewed access distribution might need a different fraction at the probationary eviction site.
- With frequency awareness now applied at both eviction sites, is there any remaining structural gap, or have the major policy improvements been captured? A direct comparison against a wider workload suite would clarify how general these gains are.
- Could an intermediate ghost window (e.g., 50% of total capacity, between probationary capacity and the failed full expansion) recover some cross-residency signal without the admission-quality problem seen at full capacity?

## Continuation

Next cycle should take v003 as the new baseline: two-segment SLRU, counter-halving demotion, frequency-aware eviction scanning the LRU quarter of protected and the LRU half of probationary. Ghost-window expansion can be deprioritized — the full-ceiling test failed and intermediate values are unlikely to move the needle meaningfully on this trace. The most productive direction is probing the probationary scan fraction (is 1/2 near optimal, or does a smaller fraction suffice?), and testing whether either or both scan fractions should vary proportionally with observed reuse-distance patterns rather than being fixed.
