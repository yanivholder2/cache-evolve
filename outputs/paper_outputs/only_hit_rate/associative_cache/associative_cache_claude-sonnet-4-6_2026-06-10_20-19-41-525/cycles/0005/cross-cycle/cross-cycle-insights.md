# Cross-Cycle Insights

## Recurring wins

- **Wider bucket geometry improves hit-rate on scan-heavy workloads.** Fewer, larger buckets give the per-set eviction policy a bigger pool of candidates, reducing premature expulsion of live items. This held from cycle 0001 onward, though returns diminish sharply beyond ~32 slots per bucket.
- **Cold-first eviction (counter == 0 preference) is the single strongest lever found.** Shielding any item with at least one access from scan displacement produced a large, durable hit-rate gain across cycles 0002–0004. The `counter == 0` boundary has been confirmed well-calibrated for workload-a.
- **Counter decay self-corrects warm-lock.** Decrementing counters only when no cold candidate exists fires rarely but reliably re-enables cold-first selection at steady state. Its targeted, infrequent nature avoids disrupting the normal eviction path.
- **LFU warm-fallback is an independently valuable axis.** Replacing positional LRU fallback with minimum-counter selection during the warm phase delivers meaningful gains, especially on narrow geometry where cold-pool exhaustion is common.

## Recurring failures

- **O(1) swap-to-back LRU optimisations corrupt recency ordering.** Any scheme that approximates LRU by swapping only boundary pointers degrades the recency gradient across intermediate slots. The hit-rate regression is reliably large; this angle is closed for ClassicLRU.
- **Pushing bucket geometry past diminishing returns.** Each successive widening step delivers a smaller fraction of the previous step's gain. Beyond ~32 slots, pure geometry expansion adds nothing measurable.
- **LFU warm-fallback after uniform counter decay is counter-productive.** Decay compresses inter-item counter differences to near-zero, making minimum-counter selection noisier than the recency order already encoded in bucket position. The two mechanisms undermine each other when combined.
- **Mid-point insertion stacked with LFU warm-fallback regresses.** Cold-first eviction is position-agnostic, so mid-point insertion's benefit (keeping new items away from the eviction front) is irrelevant to it, while the LFU fallback depends on a recency structure that mid-point insertion disrupts. These two should not be combined.
- **Threshold tightening beyond counter == 0 hurts.** Raising the warmth threshold to require ≥ 2 accesses before shielding an item prematurely evicts genuine hot-set members on workload-a, where a single access is a reliable reuse signal.

## Open questions

- **Per-op latency is unmeasured for the accumulated stack.** The combined cost of cold-first scan + counter decay pass + optional mid-point insertion has never been benchmarked. Wide-geometry vs. narrow-geometry paths may trade hit-rate for latency differently, and this comparison point remains completely open.
- **Structural generalisability beyond workload-a.** Cold-first eviction, counter decay, and the geometry saturation point have all been validated on a single LIRS scan-heavy trace. Whether these results hold under different locality profiles (Zipfian, uniform, multi-stream) is unknown.
- **Hit-rate ceiling on workload-a.** Two consecutive cycles of incremental refinements have returned regressions or neutrality, suggesting a local optimum. Whether a structurally different eviction family (CLOCK, ARC, LIRS-proper, adaptive two-queue) can push past it is unprobed.
- **Stacking counter-decay with LFU warm-fallback (non-decayed).** These two mechanisms are orthogonal in purpose — one restores cold supply, the other improves warm-phase selection quality — but have only been tested in combination after decay already compressed counters. Whether LFU fallback adds value on top of decay when decay is infrequent (wide-geometry, rarely triggered) is open.
- **Cold-pool exhaustion dynamics over long traces.** As the cache warms, the cold pool thins. Whether the decay fallback stabilises the cold/warm boundary indefinitely or causes oscillation over very long traces has not been tested.

## Per-policy notes

- **ClassicLRU (baseline):** Narrow 16-slot bucket geometry was a real bottleneck; strict O(n) recency ordering is load-bearing and must be preserved. Per-op latency evaluators for this policy have never been enabled; structural changes (e.g., timestamp removal) were tested without their intended measurement dimension running.
- **Cold-first eviction family (cycles 0002–0005):** Depends on wide-bucket geometry as a prerequisite to maintain a viable cold pool; mid-point insertion can partially substitute for geometry widening but is not fully equivalent. Counter == 0 is the confirmed optimal eviction threshold for workload-a. Counter decay is a necessary steady-state patch; without it, the cold pool exhausts and the policy degrades to plain LRU. The configuration `wide-bucket + cold-first + mid-point insertion + counter decay + counter cap` is the current best-known configuration and appears to be a local optimum on workload-a hit-rate.
- **LFU warm-fallback (cycle 0004 v002):** Effective on narrow geometry as a standalone improvement; undermined by uniform counter decay and by mid-point insertion. Should only be considered in combination with decay if decay is selective enough to preserve meaningful counter differentiation.
