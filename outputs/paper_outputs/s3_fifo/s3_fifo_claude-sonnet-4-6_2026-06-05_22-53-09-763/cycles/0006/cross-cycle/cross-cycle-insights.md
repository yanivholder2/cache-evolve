# Cross-Cycle Insights

## Recurring wins

- **Enlarging main queue at the expense of small queue.** Shrinking small from 25% → 10% of capacity (cycle 0001) was the largest single gain across all cycles; transferred capacity directly raises hit rate by reducing churn on the hot set.
- **Tighter ghost window (main_size/2 cap).** Halving the ghost cap improved hit rate independently of small-queue sizing (cycle 0002), because this workload's re-use distances fit inside the shorter window and the excess history only added noise. The gain held when stacked with the enlarged main queue.
- **Raising MAX_FREQ ceiling (3 → 4 → 5).** Each upward step produced a real but shrinking positive: a non-trivial tail of hot items legitimately earns and uses the extra retention pass. Each increment is worth attempting once; diminishing returns set in quickly.
- **Move-to-front as the primary recency-sorting mechanism.** Removing it caused the largest regression seen in any cycle (cycle 0005), confirming it is the dominant structural lever for recency within main, not merely an incidental implementation detail.

## Recurring failures

- **Loosening promotion admission (freq > 0).** Tried three times across cycles 0001, 0003, and 0004 — neutral at best, harmful in combination with ghost precision. The narrow small queue prevents a quality flood only when the threshold is held at freq > 1; relaxing it dilutes main with single-access items.
- **Expanding ghost queue beyond main_size/2.** Wider ghost coverage in cycle 0001 introduced incidental re-accesses that bypassed the small-queue filter; the workload does not carry re-use signal at longer horizons.
- **Compressing ghost queue below main_size/2.** Crossing this floor in cycle 0003 discarded genuine re-use signal; the workload's re-use distances extend into the 461–922-eviction range and require at least this window.
- **Ghost re-admission with a head-start credit (freq = 1 on re-entry).** Tried in cycle 0004; displaces incumbents that earned their slots through repeated accesses, netting a loss. Ghost confirmation of re-use potential does not justify a residency advantage over established residents.
- **CLOCK-style in-place eviction scan (removing move-to-front).** Tried in cycle 0005; the implicit positional re-ordering provided by move-to-front is load-bearing — any change that eliminates it causes a large regression.
- **Fine-grained positional manipulation within main.** Saturation-triggered head repositioning and midpoint insertion (cycle 0006) are both inert: the conditions required to benefit — a hot item sitting near the eviction frontier — almost never occur given how move-to-front already keeps such items safe.

## Open questions

- **Conditional move-to-front**: could repositioning only items above a frequency threshold improve on unconditional repositioning, or does partial recency sorting degrade to a variant of the cycle-0005 failure?
- **Phase-sensitive / adaptive mechanisms**: with all static parameter knobs effectively exhausted, do workload phase shifts (scan bursts, hot-set replacement) create windows where adaptive queue sizing or burst detection could yield gains that static tuning cannot capture?
- **Alternative algorithm comparison**: the current S3-FIFO configuration may be at a local optimum for workload-a; a head-to-head benchmark against a structurally different algorithm (e.g., TinyLFU, ARC) would clarify whether further gains require a new design rather than continued refinement.

## Per-policy notes

- **S3-FIFO (the only policy evaluated so far):** The three-queue structure has been mapped thoroughly across six cycles. Best-known static configuration: small queue ≈ 10% of capacity, main queue takes the remainder, ghost cap = main_size/2, promotion threshold = freq > 1, MAX_FREQ = 5. Move-to-front within main is non-negotiable. The ghost queue's value is precision (exact re-use signal within a bounded horizon), not coverage. All threshold and sizing knobs are at or near their effective limits; the remaining search space lies in dynamic or structural changes rather than parameter tuning.
