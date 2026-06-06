## Recurring wins

- **Eviction-side structural changes consistently pay off.** SLRU segmentation (cycle 0001), probationary victim scanning (cycle 0004), and sketch data-quality improvements (cycles 0006–0007) have all produced real gains; each restructured *how residents compete after admission* rather than when they are admitted.
- **Sketch data quality is an independent, stackable axis.** Conservative CMS update (cycle 0006) and wider sketch rows (cycle 0007) each reduced distinct error sources — estimate inflation vs. collision noise — and their gains were additive rather than redundant.
- **Promotion gating with a `was_protected` bypass is a safe pairing.** Blocking low-frequency newcomers from protected while exempting capacity-overflow demotion victims (cycles 0003, 0005) avoids penalizing hot items caught in temporary churn. The gate is correctly calibrated for the current sketch quality and does not need adjustment.
- **Baseline-reset experiments have surfaced new axes.** Twice (cycle 0004, cycle 0006) a variant rooted at the original baseline outcompeted variants built on the accumulated lineage, indicating that interaction effects between successive changes can obscure individual contributions.

## Recurring failures

- **Sketch aging/reset is consistently harmful.** Halving counters on a periodic cadence eroded frequency signal at both short intervals (cycle 0001) and long intervals (cycle 0002). On this Zipfian trace, accumulated counts are signal; there is no stale-counter problem to solve.
- **Ghost-set re-admission degrades hit rate on Zipfian.** Tested twice (cycles 0003, 0004) and regressed both times. Items evicted by TinyLFU are genuinely cold; fast-tracking them into protected displaces legitimately hot residents. This mechanism is closed.
- **Capacity-consuming admission structures are immediately penalized.** Reserving a window segment for newcomers (cycle 0001) shrank the main resident pool before any newcomer benefit could materialize. The workload does not produce items that accumulate frequency fast enough inside a fixed window to recoup the capacity cost.
- **Protected-segment victim scanning is a dead end.** The PROMO_THRESHOLD gate pre-filters protected entrants so tightly that the LRU tail of protected has a narrow frequency spread; the scan almost never overrides the default LRU choice (cycle 0006). This axis is closed.
- **Split-ratio tuning beyond 90/10, scan-window widening beyond k=4, put-path promotion symmetry, hash-seed diversification, and admission-floor tightening** all produced flat or negative results and are definitively exhausted.

## Open questions

- Does increasing sketch **depth** (number of hash rows) reduce minimum-estimator variance in a way that width cannot, or have both dimensions converged to the same collision floor?
- Does the aging/reset interval interact with the now-narrower collision profile — does faster forgetting expose more or less residual noise at the current sketch size? (This is a narrow aging question distinct from the periodic-halving variant already ruled out.)
- At what sketch quality level does the remaining hit-rate gap stop being attributable to estimation error, and does that shift the frontier back to structural eviction policy?
- Is frequency decay policy — a gradual, non-halving decay that preserves rank ordering while reducing absolute magnitude — meaningfully different from the periodic reset already shown harmful?
- Do the conservative-update and wide-sketch gains hold on non-Zipfian workloads, or are they specific to the heavy-tail concentration of this trace?

## Per-policy notes

- **SLRU 90/10 split:** Baseline since cycle 0002. Split-ratio axis is closed; the hot working set fits comfortably within the protected budget and no ratio tested beyond 90/10 has helped.
- **Probationary segment (k=4 victim scan):** The scan is effective and the window-width axis is saturated. Ghost-set fast-tracking into probationary is harmful and closed.
- **Protected segment:** LRU-tail is frequency-homogeneous after PROMO_THRESHOLD gating; internal victim scanning adds nothing and is closed.
- **PROMO_THRESHOLD=8 with `was_protected` bypass:** Compatible with k=4 scan but not synergistic; the gate collapses toward unconditional promotion in practice because protected items either exceed the threshold via get-volume or carry the bypass flag. No further threshold adjustment is warranted.
- **CountMinSketch (conservative update + 8× width):** These are the two confirmed wins in the sketch layer. Depth and aging interval are the remaining unexplored axes. Hash-seed diversification produced a marginal regression and is closed.
- **TinyLFU admission gate:** Stable across all cycles. Frequency-floor tightening is closed. Sketch-aging (the reset mechanism behind the gate) is consistently harmful at all tested cadences.
