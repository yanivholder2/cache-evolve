## Recurring wins

- Changes with a clear mechanism are the only ones that pay off reliably. Examples: removing redundant linear scans (Cycle 3) and hash-index membership (Cycle 4). Both cut put/get cost and leave the hit/miss sequence unchanged.
- Constant-only tweaks to the small queue gave a modest gain early on (10%, then 5%). Returns are diminishing and the curve is flattening.
- Keeping the small queue as a strict one-hit-wonder filter, with baseline promotion, protects hit rate.

## Recurring failures

- Anything that loosens main admission lowers hit rate. This has happened four cycles running. Examples: a lower promotion threshold, a larger small queue, recycling main evictions into the shared ghost, a freq head start for ghost re-entry, and a longer ghost history.
- Ghost-side tuning of main-evicted keys (recycling, head start, size) has repeatedly regressed or shown no gain. Cycle 2's apparent gain is now doubtful.
- Rotation caps and other bounds on already-saturated counters trade hit rate for nothing.
- Latency is a noisy selection signal. Variants with no hot-path mechanism often "won" on latency shifts alone, so regressions in hit rate got promoted. Credit latency only when a mechanism explains it.
- Refactors that delete tests, and index structures whose memory cost is hidden from the per-entry metadata figure, create hidden debt.
- Infrastructure failures (such as a Maven 429) can silently hide an idea. Retry the idea rather than treating it as rejected.

## Open questions

- Do the small-ratio, ghost and index results hold across other traces, cache sizes and repeated runs? All evidence so far comes from one trace at one size.
- What is the index's true whole-cache memory cost, and is it acceptable?
- Can the put tail be cut by making eviction and ghost trimming cheaper?
- Is any special handling of main-evicted keys worthwhile? It would need an O(1) tagged ghost and evidence of faster reuse.
- Can the tests deleted in the scan refactor be restored without losing its gains?
- Is 5% really better than 10%, or is the difference within noise?

## Per-policy notes

- S3-FIFO (the only policy tried so far):
  - Base: scan consolidation plus hash-index membership. The small ratio is 10%, or 5% if confirmed on other traces.
  - Baseline promotion threshold and baseline ghost sizing look near-optimal on workload-a.
  - Stop sweeping the small ratio, the ghost size and the promotion threshold.
  - The next gains are likely in put-tail cost (eviction and ghost trimming), validated on multiple traces and judged on hit rate.
