## Recurring wins

- Constant-only tweaks that add no work and leave metadata unchanged (e.g. a smaller small-queue ratio) are the only changes with a plausible, modest hit-rate gain so far.
- Only one cycle so far; no pattern has yet been confirmed across multiple cycles.

## Recurring failures

- Large per-op latency drops with no mechanism behind them are run-to-run noise. They let weak variants win, so latency alone is an unreliable fitness signal.
- Sharing the ghost queue between main evictions and small-queue demotions (ghost recycling) lowers hit rate. It dilutes the one-hit-wonder filter.
- Infrastructure failures (a Maven 429 on the Caffeine jar download) can discard a candidate without testing its idea. Don't read such a rejection as evidence against the idea.

## Open questions

- Is the latency signal reproducible, or noisy enough to promote regressions? Repeated runs are needed.
- Is the best small-queue ratio workload-dependent? Sweep around 10% (e.g. 5% and 15%).
- Does the lower promotion threshold (freq>=1) help? It has never been evaluated.
- Can ghost recycling work if redesigned, with a separate ghost or without the freq=1 head start?
- Do gains generalize beyond one trace (LIRS workload-a) and one cache size?

## Per-policy notes

- S3-FIFO:
  - A 10% small queue is the current best-known tweak. Build on it.
  - The promotion-threshold idea is untested and should be retried.
  - Ghost recycling of main evictions should be dropped unless redesigned.
  - All evidence comes from a single trace and cache size.
