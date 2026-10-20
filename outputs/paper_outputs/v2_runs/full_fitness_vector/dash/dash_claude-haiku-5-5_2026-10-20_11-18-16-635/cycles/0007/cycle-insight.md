# Cycle 0007 Insight

## Cohort outcome

The cohort tried three changes around the full-scan lowest-counter victim choice: an exact early exit from the victim scan on a zero counter (v001), halving every counter after each eviction (v002), and porting the saturating counter and full-scan victim choice onto the packed `Item<u64,u64>` record (v003). Only v003 showed a hit-rate gain, and it was partial. v001 and v002 have no helped dimensions, and all three are labelled Winners anyway.

## Patterns

- The full-scan, non-decaying counter remains the working base. Exiting the scan early (v001) left hit rate and metadata identical, so the victim choice was preserved. Decaying the counters (v002) lowered hit rate, which suggests the never-decaying counter is doing the retention work rather than just shielding stale residents.
- v003 is the first place the counter-based victim choice has been combined with a packed record. Hit rate rose, but by less than the hypothesis targeted, and metadata size did not change. Layout and replacement look separable again.
- Put cost still is not explained by scan length. The early exit did not make puts cheaper, which is the third cohort in a row where a shortened or cheapened victim search failed to lower put latency. The decay pass added put-path work and was the worst put result in the cohort, so the put-side cost appears to come from bookkeeping that runs on the eviction branch, not from the scan alone.
- Per-op movements again outrun their mechanisms. v001's get regression sits on a path the change does not touch, and v003's small get and put improvements are attributed to noise rather than the change.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace check first requested in cycle 0004 has now been deferred for four cycles.

## Open questions

- Does the v003 counter-plus-packed-layout hit-rate gain hold on a second, non-workload-a trace?
- Is the counter stored in spare bits or in the packed `u8` slot? The overlay source was not inspected, so this is unconfirmed, and it bears on the get and promotion decode cost raised in cycle 0006.
- Why are v001 and v002 Winners when they record no helped dimensions? The Winner basis is still not visible in the insights.

## Continuation

Next cycle should keep the full-scan, non-decaying counter as the victim-selection base and treat the packed-record port (v003) as the candidate to build on, but only after running the second-trace check that has been outstanding since cycle 0004. Drop counter decay (v002) and the early-exit scan (v001). Do not credit any per-op gain from this cohort until it is re-measured.
