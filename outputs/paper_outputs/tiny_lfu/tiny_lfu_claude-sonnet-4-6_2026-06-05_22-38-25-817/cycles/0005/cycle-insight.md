# Cycle 0005 Insight

## Cohort outcome

All three variants pushed against the edges of the cycle-0004 winner — combining promotion gating with the victim scan, widening the scan window, and enlarging the protected segment. Only the combined promotion-gate variant (v001) was selected, and then by a vanishingly small margin; the other two produced no measurable change.

## Patterns

- **Promotion gating and victim scanning are compatible but not synergistic.** Layering PROMO_THRESHOLD + `was_protected` onto the k=4 scan did not produce additive gains — on this Zipfian trace most probationary items either accumulate enough frequency to clear the threshold quickly, or carry the `was_protected` flag and bypass the gate anyway, so the gating collapses toward unconditional promotion in practice.
- **The victim-scan window is fully saturated at k=4.** Doubling to k=8 changed no eviction decisions, confirming that the coldest reachable candidate already appears within the first few LRU-tail positions. The scan's bottleneck is not window width.
- **The SLRU split-ratio axis is conclusively closed.** The gain trend from 80/20 → 90/10 does not continue to 95/5; the hot working set fits comfortably within the existing protected budget and the additional headroom rescues nothing.
- **Three simultaneous null/marginal results signal a local optimum.** The current design — 90/10 SLRU, k=4 victim scan, PROMO_THRESHOLD gate, `was_protected` bypass, TinyLFU admission — has exhausted all parametric tuning axes that have been probed across five cycles.

## Open questions

- Is frequency-ordered eviction *within* the protected segment (replacing its internal LRU order) a meaningful axis that has not yet been tested?
- Could tuning the TinyLFU admission gate itself — rather than post-admission segment management — provide the next increment, given that the segment internals are now near-optimal?
- Does the marginal win from v001's promotion gate hold at different cache-size-to-working-set ratios, or is it purely an artifact of the tight capacity used in this benchmark?

## Continuation

Adopt v001 (90/10, k=4 victim scan, PROMO_THRESHOLD≥8, `was_protected` bypass) as the new baseline, but treat all five tuning knobs as fixed — none warrant further adjustment. The next cycle should pivot to a qualitatively different structural change: the most promising unexplored axis is frequency-aware ordering within the protected segment (evicting the cold tail of protected rather than the LRU tail). If that also disappoints, the admission gate and sketch configuration are the remaining structural frontier. All prior closed axes — ghost-set, put-path symmetry, scan-window widening, split-ratio changes — should not be revisited.
