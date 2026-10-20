## Recurring wins

- **Structural compaction of node layout pays compound dividends.** Reducing node size (u32 sentinel links, bit-packing queue membership into existing fields) improves cache-line utilization on the get path and reduces memory pressure — without sacrificing eviction semantics when done carefully.
- **Behavioral/algorithmic changes outperform micro-optimizations.** Across both cycles, meaningful policy changes (two-queue promotion, MRU-tail guard) delivered measurable wins; low-level code hints (`#[inline(always)]`, API tricks) did not.
- **Get-path improvements are real but workload-specific.** Optimizations that reduce unnecessary work on the hot read path (skipping no-op promotes, better cache-line fit) consistently show up in p50 get latency.

## Recurring failures

- **Forced inlining and API-level micro-optimizations backfire.** `#[inline(always)]` can push hot paths out of instruction-cache sweet spots; `entry()` API savings are erased by borrow-checker constraints on the dominant eviction path. Both cycles confirm low-level hints are net negative here.
- **Algorithm complexity adds hidden per-operation cost.** Two-queue bookkeeping, CLOCK hand sweeps, and extra pointer work each introduced latency overhead that eroded structural gains. Simpler structural changes have consistently outperformed complex algorithmic ones.
- **Put-path improvements do not follow from get-path improvements.** Every cycle has seen asymmetry: changes that reduce get latency often have neutral-to-negative effects on put latency (MRU guard raising put p50, CLOCK sweeps blowing up put, cast overhead in sentinel links).

## Open questions

- **Can 2Q and the MRU-tail guard be composed safely?** Both are structurally compatible and target different bottlenecks; composing them is the highest-priority thread for the next cycle.
- **Is the MRU-guard p99 regression inherent?** The branch-predictor disruption raised get p99 even as p50 improved — worth probing whether branch placement or workload shape can recover the tail without losing the median win.
- **How much further can node size be reduced?** The current 24-byte node is compact, but it is unclear where diminishing returns begin; an experiment pushing toward a single-cache-line node remains unexplored.
- **Can key-clone cost on eviction be eliminated?** This is the remaining identifiable per-put cost not yet addressed, and it sidesteps the borrow-checker obstacles that killed the `entry()` approach.
- **Is CLOCK recoverable with lazy or segmented sweeps?** The hand-sweep cache-line pollution may be avoidable; if so, CLOCK's structural benefits could be revisited.

## Per-policy notes

- **LRU (baseline):** Exact ordering continues to be valuable for this workload — coarser recency signals (CLOCK's single bit) hurt hit rate, suggesting meaningful reuse heterogeneity that LRU exploits.
- **Two-queue (2Q):** Confirmed winner in cycle 0002 when layered on the u32-sentinel baseline without node growth (queue bit packed into `prev`). Hit-rate improvement is real. Composition with MRU guard is the next natural step.
- **MRU-tail guard (get-path shortcut):** Winner in cycle 0002 on p50 get, but raises p99 and put p50. Best understood as a complementary optimization to 2Q rather than a standalone policy.
- **CLOCK:** Structural compaction benefit (flat slots) was negated by hand-sweep cache-line pollution. Deprioritized unless a mechanism to avoid sweep pollution is proposed.
