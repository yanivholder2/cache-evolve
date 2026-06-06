## Recurring wins

- **Reallocating capacity to main beats loosening admission.** The one confirmed win (small queue 25%→10%) improved hit rate purely by giving main more resident slots, without touching any admission criterion. This suggests the cache is main-starved, not admission-starved.

## Recurring failures

- **Relaxing admission thresholds degrades main-queue quality.** Both attempts to widen entry into main — lowering the promotion frequency threshold and extending the ghost window — hurt hit rate. Second-chance eviction inside main cannot compensate for a noisier inflow fast enough to protect hot items.
- **Longer ghost windows add noise, not signal.** Extending ghost history captures distant, non-recurring accesses that bypass the small-queue filter without representing genuine re-use. Precision of ghost tracking matters more than coverage.

## Open questions

- Is 10% the floor for small-queue size, or do further reductions (e.g., 5–8%) keep paying off? Does the answer change under scan-heavy or bursty workloads?
- Does the optimal ghost queue size lie *below* the current baseline cap? One cycle's evidence suggests shrinking it is worth testing.
- Does the winning small=10% configuration shift the optimal promotion threshold, or does `freq > 1` remain correct regardless of small-queue sizing?

## Per-policy notes

- **S3-FIFO:** The small→main→ghost structure is sensitive primarily at the capacity split, not the admission logic. The second-chance mechanism in main is a lagging corrective, not a robust filter — upstream admission quality (frequency threshold, ghost window precision) must do the heavy lifting. Confirmed winning direction: grow main by shrinking small; do not relax promotion criteria without a compensating guard.
