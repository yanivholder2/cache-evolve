## Recurring wins

- Keeping the small queue's one-hit-wonder filter intact is what pays off. The 10% small queue with the baseline promotion threshold has held up as the best point so far.
- Giving main-eviction history its own bounded ghost (separate from the small-queue demotion ghost) is the only change that has shown a real, modest hit-rate gain. It cost some metadata and put latency.

## Recurring failures

- Changes that let more one-hit-wonders into main, or take capacity from main, lose hit rate. This covers a looser promotion threshold (freq>=1) and a larger small queue (15%).
- Sharing one ghost between small-queue demotions and main evictions dilutes the filter and lowers hit rate.
- Latency is not a usable selection signal. Large per-op latency shifts keep appearing on edits that add no hot-path work, so regressions have likely been promoted through noise.
- Infrastructure failures (for example a Maven 429) can hide ideas. A rejection caused by infrastructure says nothing about the idea itself.

## Open questions

- Is the separate main-ghost's hit-rate gain larger than run-to-run noise, and does it hold on other traces and cache sizes?
- What ghost size is best, and does re-entry from the ghost need a freq head start? Is the metadata and put-latency cost justified?
- Is the small-ratio optimum below 10%? 5% and lower are untested.
- Can repeated runs or multiple traces separate real effects from noise?
- All evidence comes from one trace (workload-a) at one cache size, so none of it is shown to generalize.

## Per-policy notes

- S3-FIFO: Current best base is the 10% small queue, baseline promotion threshold, and a separate bounded main-eviction ghost. Next steps are to tune the ghost, sweep small ratios at 5% and below, and judge on hit rate over more traces.
- S3-FIFO dead ends: ghost recycling that shares the demotion ghost, freq>=1 promotion, and small-queue ratios of 15% or more.
