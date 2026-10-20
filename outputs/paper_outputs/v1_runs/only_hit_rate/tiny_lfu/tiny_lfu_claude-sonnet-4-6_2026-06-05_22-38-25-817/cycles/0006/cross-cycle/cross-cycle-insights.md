## Recurring wins

- **Resident-ordering / eviction-side changes outperform admission-side changes.** Every durable gain has come from changing how items are selected for eviction or promotion within segments, not from adjusting who gets admitted.
- **SLRU segmentation with a 90/10 protected/probationary split** established the performance floor; the shift from a flat LRU to a two-tier structure was the largest single step up.
- **Victim-side frequency scanning in the probationary segment (k=4)** rescued warm survivors that strict LRU would have displaced; the scan is effective because probationary sees genuine churn on this trace.
- **Promotion gating (`PROMO_THRESHOLD` + `was_protected` bypass)** reduced borderline-frequency items accumulating in protected while keeping genuinely hot overflow-demoted items safe — the bypass is essential to make aggressive thresholds safe.
- **Conservative CMS update** (cycle 0006) yielded a gain after five cycles of policy tuning, confirming that reducing sketch collision inflation was a live, independent frontier that policy changes alone could not surface.

## Recurring failures

- **Sketch aging / counter halving** has regressed at every tested cadence (short in cycle 0001, 10× longer in cycle 0002). Accumulated frequency counts are signal on this stable Zipfian trace, not noise; periodic erasure harms more than it helps.
- **Ghost-set / victim-cache re-admission** regressed or produced no gain in both isolated tests (cycles 0003–0004). Ghost-ring entries are items that already lost the TinyLFU admission contest; fast-tracking them into protected displaces legitimately hot residents. Failure mode is well-understood — closed.
- **Admission window (newcomer grace period)** shrank the main resident set, and the capacity penalty was immediate; the trace does not produce items that accumulate frequency fast enough inside the window to recoup the cost.
- **Victim-scan mirrored into the protected segment** produced zero movement in cycle 0006. The `PROMO_THRESHOLD` gate pre-filters entrants, leaving the LRU tail of protected with a narrow frequency spread; the scan rarely finds a colder candidate than the default LRU head. Closed.
- **Split-ratio tuning beyond 90/10** yields no further gain (95/5 tested in cycle 0005); the hot working set fits comfortably in the existing protected budget.
- **Scan-window widening (k=4 → k=8)** changed no decisions; the coldest reachable candidate already appears within the first few LRU-tail positions.
- **Put-path promotion symmetry** had no effect on this read-dominant trace; hot items accumulate enough frequency via get() before any put() trigger could fire.
- **Admission floor increase** was redundant on a warm trace — the freq-1 vs. freq-0 edge case it targets is a cold-start artifact.

## Open questions

- Do sketch **width, depth, or reset interval** offer further signal-quality improvements now that conservative update is baseline, or is sketch configuration already sufficient?
- Does the conservative-update gain hold on **non-Zipfian workloads** (e.g., uniform or bursty access patterns), or is it specific to the heavy-tail concentration of this trace?
- Could **frequency-aware ordering within the protected segment** (replacing its internal LRU with a frequency-ordered structure) move the needle where victim scanning in protected failed?
- How does the conservative-update interaction with the **aging/reset mechanism** (if one is ever re-introduced) affect gain — does sharper estimation prolong or shorten the useful life of accumulated counts?
- At different **cache-size-to-working-set ratios**, do the promotion gate and victim scan retain their character, or are they artifacts of the tight capacity used in this benchmark?

## Per-policy notes

- **TinyLFU admission gate:** handles one-hit-wonders effectively out of the box; admission floor adjustment is redundant on warm Zipfian; the gate threshold itself has not been tuned and remains an unexplored axis.
- **SLRU segmentation:** 90/10 is the confirmed optimum split for this workload; further split-ratio adjustments in either direction are closed. The two-tier structure is a stable, high-value foundation.
- **Probationary segment:** the primary arena for victim-scan benefit; churn is real and k=4 scans consistently override LRU in meaningful cases. Scan window is saturated at k=4.
- **Protected segment:** frequency-homogeneous at the LRU tail after promotion gating; victim scanning inside protected is ineffective and should not be revisited. Internal ordering changes (frequency sort) remain the one untested axis.
- **CountMinSketch:** standard update inflates estimates via hash collisions and suppresses correct eviction decisions; conservative update is the new baseline. Width/depth/reset are untuned and are the highest-priority sketch axis for future cycles.
- **Promotion policy:** `PROMO_THRESHOLD` (≥ 8) with `was_protected` bypass is compatible with victim scanning but not synergistic — gating collapses toward unconditional promotion in practice because frequently accessed items clear the threshold quickly and demoted hot items carry the bypass flag. The combined design is safe but yields only marginal additive benefit.
