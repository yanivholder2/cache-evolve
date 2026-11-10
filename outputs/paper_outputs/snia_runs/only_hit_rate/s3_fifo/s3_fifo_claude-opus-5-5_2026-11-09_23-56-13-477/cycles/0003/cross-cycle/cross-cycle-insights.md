## Recurring wins

- The baseline strict two-reuse graduation rule plus a freq reset on graduation has survived three cycles. No threshold, split, gate or credit variant has beaten it on every trace.
- The winning variant in every cycle has been the one that regressed least, not one that clearly improved. Policy gains have been marginal, and the policy looks like it has plateaued on these traces.
- Of all the policy levers, stricter admission into a full main is the only one that has produced a clear gain. Which keys get into main matters more than how big the queues are.
- The O(1) index/slab refactor has kept hit/miss behaviour identical twice (cycles 1 and 3, the second time enforced by an oracle test). It is a safe structure to build further policy work on.

## Recurring failures

- Changing one constant at a time (graduation threshold, static small/main split) splits the traces: hm_0, mainly at the small size, gains, while web_0 and hm_0 at the large size lose. The early "hm_0 wants a looser threshold" result came from cold-start or small-cache conditions.
- Adaptive signals that go flat or are biased have failed. Main occupancy saturates, so gating on it falls back to the baseline. Ghost-return rate is inflated by the ghost being bigger than the small queue, so it turned lenient on exactly the traces where strictness helps.
- Rewarding the ghost path in any way (loosening admission on ghost returns, giving returnees credit) costs hit-rate. In steady state the ghost mostly holds weak single-reuse keys.
- Extra protection inside a full main (graduate credit, second chances) just evicts other useful entries. The cost of a bad admission is paid when the key enters main and can't be repaired later.
- Adaptive ARC-style splitting moved hit-rate only by about noise, with the sign flipping across traces. Without size weighting it tends to drift toward a larger small queue.
- Per-op-time and metadata-size have not been measured for three cycles running, despite being requested each time. Cost-side work is still unvalidated.

## Open questions

- How much does the O(1) structure actually cut per-op-time, and how much metadata-size does it add, compared with linear scans? This blocks any decision about cost trade-offs.
- Can per-key evidence beat the global rate proxies that have failed so far? Candidates are small-queue freq at the time a key is ghosted, or a direct comparison of the misses avoided by promoting a key against the cost of the main entry it displaces.
- Where do the adaptive variables (`small_size`, the switched threshold) settle on each trace? This has never been logged, and logging it would explain why the adaptive variants failed.
- Is the remaining hit-rate gap closable at all on hm_0/web_0, or should the remaining hypothesis budget go to reducing cost and metadata?

## Per-policy notes

- S3-FIFO (the only policy explored): the current parent is the O(1) index/slab structure with the occupancy-gated two-reuse rule and freq reset. Because main saturates, this effectively behaves like the baseline two-reuse rule.
- S3-FIFO small/main split: static values from 10% to 25% and the adaptive split have all been roughly neutral. Treat the split as a low-yield knob.
- S3-FIFO ghost queue: it is useful as a filter but not as a reward path or a global signal. It is about 3× the small queue in size, and on the linear structure each scan adds per-miss cost.
- Trace behaviour: web_0 consistently punishes admitting keys accessed exactly twice or reused at medium frequency. hm_0 tolerates looser admission only at the small cache size.
