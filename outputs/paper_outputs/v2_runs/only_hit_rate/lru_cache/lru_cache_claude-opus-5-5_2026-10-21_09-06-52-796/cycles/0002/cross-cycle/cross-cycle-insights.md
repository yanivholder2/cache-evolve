## Recurring wins

- Strict admission filtering is the only thing that has reliably raised hit rate. S3-FIFO's probation-queue filter won cycle 1, and every later change that loosened it lost hit rate. Across both cycles, deciding *which keys get in* matters more than *how recency is tracked*.
- The fixed ~10% probation (small-queue) split has held up. Drifting away from it has cost hit rate.
- Exact LRU ordering is not needed for hit rate on this trace. Coarser recency (CLOCK) matched LRU, so precise ordering can be traded for cheaper metadata without a hit-rate penalty.

## Recurring failures

- **The measurement gap has lasted every cycle.** Only hit rate has ever been reported. Most hypotheses now claim `metadata-size` or `per-op-time` wins (CLOCK, the u32/Fx representation, the fingerprint ghost ring, LIRS's costs), and none of those claims has been checked. More hit-rate-only cycles give little new information.
- **Loosening admission into `main` keeps losing.** Longer ghost windows, ghost-hit fast paths into `main`, and growing the probation queue all admitted keys without proven resident reuse, and all lowered hit rate.
- **Treating a ghost hit as a reuse signal is misleading on loops longer than the cache.** A key can come back within the ghost window but not within `main`'s residency. Policies that act on ghost hits (lengthening the window, resizing adaptively) keep getting this wrong.
- **Bundling a representation change with a policy change confuses attribution.** When both change at once, a hit-rate drop can't be pinned on either.

## Open questions

- Once measured, do S3-FIFO's ghost structures cost more per-key memory than dropping the `prev`/`next` links saves? Is `get` actually faster than LRU?
- Does a compact fingerprint ghost ring at the parent's original window size reduce memory and per-op time without hurting hit rate? Do its stale-slot and early-decrement issues matter at that size?
- Are the policy-independent representation tweaks (Fx hasher, `u32` indices, MRU fast path) truly orthogonal, so they can be stacked onto the S3-FIFO winner?
- Does CLOCK's victim sweep cause tail-latency spikes on `put` when most entries are marked referenced?
- Can LIRS with a larger HIR fraction and an O(1) non-resident FIFO close the hit-rate gap to S3-FIFO, and at what memory cost?

## Per-policy notes

- **S3-FIFO:** the best hit rate so far, and the right lineage to build on. Use the cycle-1 variant as the parent, not the weaker cycle-2 children. Keep strict admission and the ~10% split. Remaining work: compact the ghost structures at the same window size, and graft on the representation tweaks.
- **ARC-style adaptivity (applied to S3-FIFO):** the adaptive split drifted toward a larger probation queue, which let in more scan traffic and lost hit rate. Deprioritize it.
- **CLOCK:** hit rate about equal to LRU. Its only possible value is memory or latency, which has not been measured. Don't do further recency tweaks unless measurement shows a clear win.
- **LIRS:** beats LRU but trails S3-FIFO. It is heavier: a larger node and a linear walk to enforce the ghost bound. Revisit only after cost can be measured and the bound walk is O(1).
- **Strict LRU + representation tweaks:** hit rate unchanged, as intended. This is a stackable speed/memory candidate whose benefit has not been measured yet.
