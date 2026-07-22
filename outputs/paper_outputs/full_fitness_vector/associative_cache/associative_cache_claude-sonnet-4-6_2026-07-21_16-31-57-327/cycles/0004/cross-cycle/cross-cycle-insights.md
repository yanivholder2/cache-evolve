## Recurring wins

- **32-byte aligned struct is a confirmed, non-negotiable base.** Padding the compact 24-byte struct to 32 bytes eliminates cache-line straddling and halves p99 tail latency. Every cycle that dropped it regressed immediately; every cycle that carried it forward held the alignment benefit. It sits below the 40-byte baseline footprint while providing structural stability.
- **LFU-semantics ordering reliably lifts hit rate.** Frequency-based eviction consistently steers eviction toward genuinely cold items on both LIRS and Zipfian traces, across cycles 1–4. The signal is durable; the cost has only ever been on the get path, not the eviction path.
- **Early-return / skip-work guards are sound latency levers.** The position-0 guard (skip promotion when already at head) halved p99 latency by eliminating unnecessary writes. The early-break on a failed swap in the 3-swap bounded loop likewise short-circuits the common Zipfian hot-key case in one iteration. Both validate the write-elimination hypothesis.
- **Bounded promotion loops (cap ≈ 3) resolve mid-bucket convergence without per-op-time cost.** Three swaps is sufficient for mid-bucket items to reach the head zone on Zipfian workloads; single-swap was the precise bottleneck in cycle 3, and three swaps closed the convergence gap cleanly in cycle 4.

## Recurring failures

- **O(n) eviction / lookup scans reliably kill per-op-time.** Full LFU frequency scans (cycle 1) and wide-bucket linear scans (cycle 1) each regressed latency even when they improved hit rate. Any approach requiring a full traversal per access is disqualified at these bucket sizes.
- **Swap-to-back promotion is a double loss.** It neither addresses the real latency bottleneck (which is not the shift cost at small associativity) nor preserves hit quality on Zipfian workloads. Closed after cycle 2; should not be revisited.
- **Two-zone (hot/cold) partitioning is definitively closed.** Boundary instability with every insertion degrades hit rate, and the hot-zone bypass saves no real work because baseline LRU already positions hot items at low-removal-cost slots. No reformulation at these bucket sizes is worth pursuing.
- **Struct shrinkage without alignment accounting regresses latency.** Removing fields narrows footprint on paper but can alter cache-line packing in ways that raise get latency even when the removed field was never read (cycle 1). Size and performance objectives must be evaluated together.
- **Per-op-time is no longer a reliable discriminating axis.** From cycle 4 onward the benchmark's ~100ns timer floor masks real differences; variants that should diverge on latency instead report identical floor values. Further per-op-time optimisation hypotheses cannot be fairly evaluated without resolving the measurement-scale problem first.

## Open questions

- **Are the 3-swap bounded loop and always-advance unconditional promotion additive or redundant?** Both target mid-bucket convergence from different angles; composing them may yield further hit-rate gain or simply converge to the same eviction ordering (cycle 4).
- **Is the timer floor a configuration issue or a workload-scale issue?** The 100ns floor may be addressable via higher operation count or a longer trace that amortises measurement noise — or it may require a different benchmarking strategy entirely. Resolving this is more urgent than adding further structural complexity.
- **Does two-tail eviction selection compose cleanly with the aligned struct?** The two-tail selector was tested without the compact layout in cycle 4 and its benefit was erased; a clean re-test paired with the 32-byte base is still pending.
- **Can the metadata-size dimension be recovered without sacrificing alignment?** The 32-byte struct recovers the footprint win vs. the 40-byte baseline but not the full metadata-size metric. Shrinking a different field — rather than removing padding — has not been explored.

## Per-policy notes

- **LFU (frequency-ordered eviction):** Hit-rate advantage is consistent across scan-heavy and Zipfian traces; eviction is now genuinely O(1) via tail removal. The open tension is on the get path: a frequency-gate neighbour check (soft-LFU) was outperformed by unconditional always-advance in cycle 4, suggesting the gate strands recently-reused items below stale-frequency neighbours. Whether pure LFU frequency ordering or recency-biased always-advance is the better complement to the bounded loop is unresolved.
- **LRU / recency-biased promotion:** The always-advance rule (unconditional one-step recency promotion) outperformed the soft-LFU neighbour check by keeping tails genuinely cold. Two-zone partitioning failed to improve on plain LRU because LRU already positions hot items efficiently for eviction avoidance. Recency-advance remains the strongest standalone promotion heuristic confirmed so far.
- **Swap-based / structural promotion:** Swap-to-back closed after cycle 2. Single-swap-only (cycle 3) was too conservative for mid-bucket convergence on Zipfian workloads. The 3-swap bounded loop (cycle 4) is the current best structural promotion strategy and should be the default base for future variants.
