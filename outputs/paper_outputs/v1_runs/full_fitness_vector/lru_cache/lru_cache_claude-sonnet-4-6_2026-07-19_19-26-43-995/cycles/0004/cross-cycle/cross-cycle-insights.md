## Recurring wins

- **Structural compaction of node metadata pays off reliably.** The u32-sentinel link encoding (Cycle 0001) halved node size and improved get latency through better cache-line utilization; this remains the unchallenged baseline. Encoding policy bits (queue-membership flag, SEEN_BIT) into spare fields of existing fields costs nothing in layout and has consistently been the right implementation approach.
- **Meaningful behavioral changes outperform micro-optimizations.** Across every cycle, variants that made a genuine algorithmic change (2Q policy, MRU-tail guard) won or nearly won, while variants applying code-level hints (`#[inline(always)]`, `entry()` API, forced inlining) consistently failed. The instruction-cache and borrow-checker constraints in the hot path make micro-optimization fragile.
- **Two-queue (2Q) policy improves hit-rate on Zipfian workloads.** The protected-bit-in-prev encoding kept Node at 24 bytes while delivering 2Q semantics (Cycle 0002 winner). Hit-rate gains from protecting hot items have appeared across multiple cycles even when the per-op cost moved unfavorably.

## Recurring failures

- **CLOCK-style eviction has failed twice with the same root cause.** In Cycle 0001, a CLOCK variant lost due to hand-sweep cache-line pollution. In Cycle 0004, a bitmap-backed CLOCK variant lost due to Vec header bloat and the same sweep cost at demotion time. The amortization argument for deferred relinking only holds when promotions vastly outnumber demotions — on a full cache these rates are coupled, making the trade-off structurally unfavorable.
- **Single-path shortcuts with low fire rates are costly.** The MRU-tail guard won in Cycle 0002 on its native workload but failed when composed with 2Q in Cycle 0003, because it fires too rarely on workload-a to justify the unconditional check. The same failure mode appeared in Cycle 0004 with two-hit graduation: the splice was triggered on first probationary access (the common case) rather than second access (the rare graduation event).
- **Adaptive and policy-response mechanisms are wasted on a stationary distribution.** The Cycle 0004 adaptive quota sizer never triggered a single adjustment epoch on the stable Zipfian trace, yet paid counter-increment and branch overhead on every operation. Any mechanism designed for workload phase shifts is invisible to this benchmark.
- **API-level put-path tricks are blocked by borrow-checker constraints.** The `entry()` API optimization (Cycle 0002 loser) failed because the occupied `VacantEntry` must be released before `map.remove()` can run on the eviction path, erasing the promised single-probe benefit. This is a structural incompatibility, not a tuning issue.
- **Policy-layer search space around 2Q appears largely exhausted.** Adaptive quota, CLOCK second-chance, MRU-tail guard, two-hit admission, protected-quota tuning, and 2Q+guard composition have all been tried; none produced a net winner across both hit-rate and latency on this workload.

## Open questions

- **Can the two-hit graduation restructuring recover the second-chance benefit?** Marking the SEEN_BIT without relinking on first probationary access (paying only a bitwise OR) and deferring the splice to the second access would move extra work to the rare path — the right calibration. This has not been tested.
- **Do non-Copy key types finally validate the key-clone elision hypothesis?** The eviction-path refactor showed no effect under `Copy` keys. A `String`- or `Arc<str>`-keyed benchmark is needed to determine whether eliminating the clone on eviction is a real put-path win.
- **Can a structural memory-layout change reduce node-access cache-miss cost without per-op branches?** Arena packing, slab grouping, or hot-field alignment have not been tried. This is the next orthogonal axis: a structural win might reopen headroom for policy experiments that currently can't overcome the tight per-op budget.
- **Is there a workload where adaptive quota sizing earns its overhead?** On a bursty or phase-shifting trace the mechanism might actually trigger and produce a net hit-rate gain. The current benchmark's stationarity prevents answering this.

## Per-policy notes

- **LRU (baseline):** The u32-sentinel doubly-linked list is the structural baseline; 24-byte nodes. All subsequent work builds on this. Get latency is excellent when the hot path avoids extra branches.
- **Two-queue (2Q):** Confirmed hit-rate improver on Zipfian workloads. Protected-bit-in-prev encoding keeps node at 24 bytes. Protected quota at 75% of capacity is the current default; expanding to 87.5% improves hit-rate but degrades throughput proportionally. Composing 2Q with the MRU-tail guard added cumulative overhead without net gain on workload-a.
- **CLOCK:** Failed in two independent implementations (linked-list sweep and bitmap-backed). Deferred relinking does not amortize well when promotion and demotion rates are coupled at capacity. Deprioritize unless a specific mechanism to avoid hand-sweep cache pollution is proposed.
- **Two-hit / second-chance:** Directionally correct (one-hit graduates do exist in the trace), but fire-rate calibration is wrong — the extra work currently lands on the common path. Restructuring to fire only at second access is the required next step before this can be evaluated fairly.
- **MRU-tail guard:** Workload-dependent win. Fire rate too low on workload-a at steady state under 2Q. Do not revisit in isolation; only reconsider if workload characteristics change or fire-rate analysis shows a higher-hit-rate regime.
