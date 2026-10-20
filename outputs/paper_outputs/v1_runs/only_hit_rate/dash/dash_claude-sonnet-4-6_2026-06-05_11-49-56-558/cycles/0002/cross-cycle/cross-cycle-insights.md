## Recurring wins

- **Hot-tier topology expansion is the dominant lever across both cycles.** Increasing the normal/stash ratio (50/50 in cycle 1, 75/25 in cycle 2) has produced the largest and most consistent hit-rate gains. The baseline over-provisions stash relative to what this workload needs; each rebalance toward more normal buckets has paid off without regression.
- **MRU re-entry is a genuine, orthogonal gain.** Inserting items at the MRU position after a stash hit improves hit-rate independently of topology changes and stacks additively with them. The gain is secondary to topology but reliable and non-interfering.
- **Independent improvement axes do not trade off.** Across both cycles, no tested variant caused a regression; multiple levers (topology, re-entry position, eviction policy) appear safe to layer.

## Recurring failures

- **Coarse frequency-based eviction consistently underperforms or harms.** Workload-a's access distribution punishes aggressive frequency discrimination: LFU's benefit in cycle 1 was narrowly workload-contingent, and aging-LFU's symmetric per-hit decay in cycle 2 actively evicted mildly-warm items that LRU would have retained. Any frequency signal applied too broadly or too aggressively degrades performance on this trace.

## Open questions

- **Where does the normal/stash gain curve peak or break?** The curve was still ascending at 75/25 (24 normal / 8 stash). A push to 28/4 or beyond is needed to locate the optimum or identify the point where stash under-capacity starts hurting cold-item re-entry.
- **Does MRU re-entry compound with 75/25 topology?** The combination was not tested directly in cycle 2; given the additive pattern observed in cycle 1, this is the highest-priority low-risk compound to run next.
- **Can aging-LFU be rehabilitated with finer granularity?** A smaller decay increment or access-count-gated decay might capture frequency signal without draining mildly-warm items. Unknown whether any such formulation beats plain LRU on workload-a.

## Per-policy notes

- **LRU (baseline):** Default recency ordering under-serves frequency-skewed workloads when the hot tier is under-provisioned. Once topology is corrected, its remaining weakness is the re-entry position after a stash promotion.
- **LFU:** Produced a win in cycle 1 on frequency-skewed workload-a. Benefit is narrow — not expected to generalise to uniform or recency-heavy traces. Hold in reserve for workload-specific probing.
- **Aging-LFU (symmetric per-hit decay):** Actively harmful in its current form on workload-a. Co-resident mildly-warm items are collateral damage of each cache hit, leading to premature eviction of useful items. Do not use without a fundamentally gentler decay mechanism.
- **MRU re-entry (insertion-position change):** Consistent secondary gain; reduces the time a newly promoted item spends below promotion eligibility. Marginal value diminishes as the normal tier grows (more dwell time for all items), but it remains non-negative and worth stacking.
