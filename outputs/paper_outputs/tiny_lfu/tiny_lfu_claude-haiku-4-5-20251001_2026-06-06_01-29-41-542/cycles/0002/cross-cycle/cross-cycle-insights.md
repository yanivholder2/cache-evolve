# Cross-Cycle Insights

## Recurring wins

- **Sketch parameter tuning (width, depth) consistently improves hit rate**: Cycle 0001 established that wider sketches reduce collision errors; Cycle 0002 confirmed depth also helps. Both are orthogonal improvements with measurable but declining returns.
- **Precision beats permissiveness**: Both cycles showed that improving measurement accuracy (wider/deeper sketches) outperforms changing selection logic (decay, threshold relaxation). The baseline admission logic is sound; the bottleneck is frequency estimation noise.
- **Strict admission gates outperform relaxed ones**: Cycle 0001's v003 relaxation from `>` to `>=` hurt hit rate; even with improved frequency estimates in Cycle 0002, tightening (not loosening) logic paid off.

## Recurring failures

- **Frequency decay removes valuable signal**: Cycle 0001 v002 failed when halving counters periodically; early-earned frequencies remain predictive and monotonic accumulation preserves them better than resets.
- **Threshold relaxation to handle ambiguity backfires**: Cycle 0001 v003 showed that allowing tied estimates to evict hot residents harms precision. Uncertainty should favor stringency, not inclusion.
- **Sketch parameter space saturates rapidly**: Width showed diminishing returns (0.0047 gain → 0.0006 gain across cycles); combined tuning of width and depth yields modest composite gains. Sketch-only refinement has ceiling.

## Open questions

- Are other sketch parameters (counter saturation strategy, hash seeding) worth pursuing before abandoning Count-Min altogether?
- Can non-frequency signals (age, size, access pattern clustering) unlock larger gains than further sketch refinement?
- Does the strict admission gate logic remain optimal with substantially improved frequency estimates, or should selection criteria expand?
- Is LRU ordering still the right eviction order given better frequency discrimination, or would LFU/hybrid approaches benefit?

## Per-policy notes

- **Count-Min sketch with strict gate (Cycle 0001-0002 focus)**: Width and depth tuning are productive but plateau within two cycles. Frequency-precision surface has been substantially refined; further diminishing returns from sketch parameters alone. Next leverage likely outside Count-Min (alternative frequency structures, non-frequency signals, or eviction ordering).
