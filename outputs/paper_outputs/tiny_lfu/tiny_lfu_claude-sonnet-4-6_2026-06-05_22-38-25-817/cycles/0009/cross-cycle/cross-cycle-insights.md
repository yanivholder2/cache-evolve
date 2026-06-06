# Cross-Cycle Insights

## Recurring wins

- **Eviction-side ordering beats admission-side filtering.** Across the full lineage, changes that restructure how already-admitted items are ordered and protected (SLRU segmentation, k=4 victim scan, frequency-ordered demotion) have outperformed changes that intervene at the admission gate (aging cadence, newcomer windows, admission floor). The one major exception — PROMO_THRESHOLD — works because it acts as a post-admission *promotion* gate, not a true admission filter.
- **Sketch signal quality amplifies downstream policy.** Conservative CMS update (cycle 0006) and wider sketch width (cycle 0007) both produced real gains, and recalibrating PROMO_THRESHOLD under the improved signal (cycle 0009) recovered further gain that was masked by earlier noise. Improving estimation accuracy is a prerequisite for accurate policy conclusions.
- **SLRU probationary stage is load-bearing as a timing gate.** Forcing items through probationary before protected admission consistently validates demand. Every attempt to bypass it (direct-to-protected, ghost-set fast-track) has regressed; keeping the two-stage path is structurally correct.
- **Conservative update and sketch width are complementary.** They address distinct error sources (estimate inflation vs. row collision) and their gains stack without overlap; together they pushed the sketch to a high-accuracy regime.
- **PROMO_THRESHOLD remains the highest-leverage policy knob.** It was effective across cycles 0003 and 0009 (with the `was_protected` bypass making aggressive gating safe), and its optimal value shifted meaningfully once sketch noise was reduced.

## Recurring failures

- **Sketch count aging is reliably harmful regardless of cadence.** Tested at short intervals (cycle 0001) and very long intervals (cycle 0008); periodic halving always erases more useful accumulated signal than stale noise. The workload's frequency distribution is stable enough that historical counts are signal, not noise — this axis is closed.
- **Ghost-set re-admission degrades hit rate on Zipfian.** Tested in cycles 0003 and 0004; ghost-ring candidates are precisely the items that already lost TinyLFU admission — they are genuinely cold, and elevating them displaces hot residents. The failure mechanism is well-understood; this axis is closed.
- **SLRU split-ratio tuning is exhausted.** 80/20 → 90/10 produced a marginal gain; 90/10 → 95/5 produced nothing. The hot working set fits comfortably in the protected segment at 90/10; the axis is closed.
- **Protected-segment victim scanning provides no additional benefit.** Because PROMO_THRESHOLD pre-filters entrants, the LRU tail of protected has a narrow frequency spread; the scan rarely overrides the default eviction choice. Closed in cycles 0006 and 0009.
- **Sketch depth and hash-seed diversity are non-levers.** At 8× width the per-row collision floor is already low enough that extra rows and diverse seeds provide no marginal discriminating power. Closed in cycle 0008.
- **Put-path promotion symmetry is irrelevant on this read-dominant trace.** Hot items accumulate sufficient frequency via get alone; the put trigger never fires in a consequential window.
- **Capacity-consuming admission windows impose structural penalties the trace immediately punishes.** Shrinking the main LRU for a newcomer-grace buffer costs more than it recovers before frequency accumulates.

## Open questions

- **Does further PROMO_THRESHOLD reduction (below 6) unlock additional thin frequency slices, or does protected-segment churn begin to outweigh the benefit?** The gain at 8→6 was real but thin; the inflection point has not been found.
- **Does the SLRU split ratio deserve re-evaluation now that sketch quality is substantially better?** Earlier closure was reached under noisier conditions; a different split might make better use of accurate per-item frequency estimates.
- **Is the remaining hit-rate gap attributable to irreducible workload entropy, or does a structural policy change still have room to move?** All nine cycles have been parametric or sketch-layer; no fundamentally different eviction model (e.g., frequency-LRU hybrid for protected ordering, TinyLFU admission threshold tuning) has been tested.
- **How robust are the current gains across workloads with non-Zipfian or time-varying access distributions?** All findings are from a single stable trace; the interactions between promotion gating, conservative update, and workload characteristics are unknown elsewhere.

## Per-policy notes

- **TinyLFU admission gate:** The strict inequality already handles one-hit-wonders on this trace; raising the floor to freq=2 was redundant (cycle 0006). The gate is correctly calibrated and should be treated as fixed. The sketch backing it is now fully tuned (conservative update + 8× width + 4 rows).
- **SLRU probationary segment:** 10% of capacity; acts as a timing/validation buffer. Victim scan at k=4 is the effective policy; widening to k=8 added nothing (cycle 0005). The segment's role is structural and correct; no further tuning warranted.
- **SLRU protected segment:** 90% of capacity; PROMO_THRESHOLD=6 with `was_protected` bypass is the current best setting. Frequency-victim scan within protected is ineffective due to the narrow spread left by the promotion gate (cycles 0006, 0009). The `was_protected` flag is essential — without it, capacity-overflow demotees are incorrectly penalized by a high threshold.
- **CountMinSketch:** 8× width, 4 rows, conservative update enabled, no aging. All sketch axes are closed. Conservative update is the single most impactful individual change across the lineage (cycle 0006), amplified by width increase (cycle 0007).
- **Ghost-set / victim cache:** Tested twice with different buffer sizes and integration points; consistently harmful. Permanently closed.
