## Recurring wins

- **Enlarging the main queue by shrinking the small queue** has improved hit rate in every cycle it was tested; the initial large step (25% → 10%) was the highest-yield single change observed so far.
- **Tightening the ghost queue cap** (halving it relative to baseline) improved hit rate in cycle 0002 regardless of which small-queue ratio accompanied it, confirming ghost precision as an independent, reliable lever.
- **Stacking complementary levers beats single-axis tuning.** The combination of a larger main queue and a tighter ghost cap outperformed either change alone, suggesting the two adjustments amplify each other rather than trade off.
- **Keeping the promotion threshold at `freq > 1`** has been consistently safe; every attempt to relax admission quality has hurt performance.

## Recurring failures

- **Relaxing admission quality into main** (lowering the promotion threshold or widening the ghost window) degraded hit rate every time it was tried. Second-chance eviction cannot compensate for a flood of lower-quality entries; upstream filtering is the load-bearing mechanism.
- **Extending the ghost window beyond the baseline cap** introduced noise from distant, non-recurring accesses and hurt performance; broader ghost coverage is consistently harmful on this workload.
- **Incremental small-queue-only shrinkage** past the initial jump delivers steeply diminishing returns; marginal gains from further transfers of capacity from small to main are now small.

## Open questions

- Does ghost precision continue to improve below main_size/2 (e.g., main_size/4), or does the window become too narrow and begin missing genuine re-uses? This is the most pressing next probe.
- Is there a true floor for small-queue size, or have we effectively plateaued? Testing capacity/12–capacity/14 on the current best baseline would settle this.
- With the ghost now much sharper and main-queue quality protected, is there finally headroom to revisit `freq > 0` as the promotion threshold — and if so, must it be paired with an additional guard to avoid flooding main?
- Do the winning configurations hold under scan-heavy or bursty workload phases, or are the optimal queue ratios trace-specific?

## Per-policy notes

- **S3-FIFO (main queue):** The second-chance mechanism is sensitive to entry quality and cannot self-correct once flooded; upstream guards (frequency threshold, ghost gate) are the critical path. Expanding resident capacity in main has been reliably good; the benefit appears to come from reduced eviction churn rather than raw size alone.
- **S3-FIFO (small queue):** Functions best as a narrow admission filter. The large initial reduction from the default 25% allocation was the dominant win of cycle 0001; each subsequent incremental reduction yields less. Likely near its optimal floor.
- **S3-FIFO (ghost queue):** Precision of the ghost signal matters more than its coverage. Halving the cap was beneficial; extending it was harmful. The sweet spot appears to be a window short enough to capture only high-recency re-uses, with further shrinkage still untested and potentially valuable.
