# Cycle 0002 Insight

## Cohort outcome

Three variants were tried: a lower promotion threshold (freq>=1) on the 10% small queue (v001), a 15% small-queue ratio (v002), and a separate bounded main-eviction ghost (v003). Only v003 shows a real, modest hit-rate gain. v001 and v002 were slight hit-rate regressions, so they probably won through noisy latency numbers.

## Patterns

- Both constant-level tweaks to the small-queue path (looser promotion, larger small queue) lost hit rate against the 10% parent. This points to 10% with the baseline threshold being a local optimum on this trace.
- The small queue's filtering role appears to matter more than quick promotion or extra residence time. Anything that lets more one-hit-wonders into main, or takes capacity from main, hurts.
- v003 supports the diagnosis from Cycle 1. Main-eviction recycling failed because it diluted the shared demotion ghost, and a separate ghost keeps the filter intact and gives a small positive gain. The cost is a struct-level metadata increase and a slower typical put.
- Latency shifts appeared again in all three variants, including edits that add no hot-path work. Every variant insight attributes them to noise. Latency is still not a trustworthy selection signal.
- All evidence still comes from one trace (workload-a) at one cache size.

## Open questions

- Is v003's hit-rate gain larger than run-to-run noise, and does it hold on other traces and sizes?
- Is the 25% main-ghost size a good choice? Is it worth the metadata and put-latency cost?
- Is the small-ratio optimum below 10% (the 5% point was not tested this cycle)?

## Continuation

Treat v003's separate main-eviction ghost as the leading direction and tune it: ghost size, and whether it needs a freq head start on re-entry. Drop the freq>=1 promotion threshold and larger small-queue ratios (15% and up), since both regressed. If anything is swept further, test 5% and below. Evaluate on more traces and repeated runs, and judge on hit rate rather than latency.
