## Recurring wins

- None established yet. Only one cycle is in the ledger, and no variant clearly beat baseline on hit rate.
- Tentative (cycle 0001 only): more main-queue residency or main-victim history tends to help hm_0. Treat this as a lead to recheck, not a proven win.

## Recurring failures

- Single-axis capacity/history tweaks (fixed small ratio, ghosting main victims, adaptive split) only move the trade-off around the trace×size grid. hm_0 and web_0-large respond in opposite directions, so the summary barely changes.
- Any change that shrinks the probation window, or the ghost space meant for small-queue demotees, hurts web_0 at large cache sizes.
- Per-op-time comparisons are unreliable so far. Every variant showed a speedup its diff can't explain, including ones that only add work, which points to a shift in the benchmark environment.

## Open questions

- Ghost capacity looks like a hidden coupled variable. Could sizing it independently of `main_size`, or splitting it by source (small demotees vs. main victims), keep the hm_0 gain without the web_0-large loss?
- Can adaptive small/main sizing work with balanced signals (normalized by queue size, or adjusted once per main eviction), a ghost that tracks main's live size, and smaller steps at large capacity?
- Is the timing drift caused by the baseline running in a different environment? Re-run the baseline on the same machine before giving any variant timing credit.

## Per-policy notes

- S3-FIFO: the small/main split and the ghost bound are coupled. Changing one while holding the other fixed caused mismatch losses in two of three variants.
- S3-FIFO: metadata per entry stayed flat across all variants tried. Only adaptive sizing added a small fixed per-cache cost.
- S3-FIFO: fixed-ratio sweeps are not worth repeating. Put future effort into ghost capacity as its own variable.
