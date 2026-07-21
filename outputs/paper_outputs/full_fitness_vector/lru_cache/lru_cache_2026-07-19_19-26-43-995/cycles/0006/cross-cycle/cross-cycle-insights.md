# Cross-Cycle Insights

## Recurring wins

- **Structural node compaction pays off consistently.** Reducing node size — via u32 sentinel links (Cycle 0001) and the link-array split separating metadata from key/value (Cycle 0005) — improved cache-line density and get latency without introducing per-op regression. Each compaction that preserved the LRU chain outperformed compaction that eliminated it.
- **Spare-field encoding enables policy complexity for free.** Folding queue-membership state into an existing spare pointer bit (Cycle 0002: protected-bit in `prev`) achieved full 2Q semantics at zero layout cost. This pattern generalised: any flag that fits in an existing spare bit is worth attempting before widening the struct.
- **The 87.5% protected quota is the settled 2Q optimum.** Tested across Cycles 0005 and 0006 on the link-array baseline, the quota enlargement from 75% to 87.5% reproducibly improved hit-rate with no measurable per-op-time cost. This is now the confirmed policy baseline.
- **"Do no harm" is the winning bar on this workload.** The per-op budget is tight enough that the bar for a winner is cost-neutrality, not positive latency gain. Changes confirmed to be cost-neutral should be accepted as structural progress even when latency benefit is below the measurement floor.
- **Simpler behavioral changes beat layered micro-optimizations.** Across every cycle, the variant with the fewest added branches or indirections outperformed more elaborate compositions (Cycles 0001, 0002, 0004).

## Recurring failures

- **CLOCK-style deferred relinking has failed twice and should not be revisited.** Both a pointer-chased variant (Cycle 0001) and a bitmap-backed variant (Cycle 0004) lost due to sweep-induced cache-line pollution and layout overhead. The underlying trade-off is unfavorable: amortising relinking only pays when promotions vastly outnumber demotions, but on a full cache the rates are coupled.
- **SEEN_BIT two-hit admission failed across three consecutive cycles (0004–0006) and is retired.** Each incarnation — full splice on first hit, cost-free bit-OR, composition with quota enlargement, eviction-path rescue — exposed the same root cause: workload-a does not generate enough within-window repeat accesses for a two-hit gate to be net beneficial. The rescue variant compounded the problem by unconditionally clearing the bit on requeue.
- **Composition of adjacent-cycle winners consistently fails.** Layering two independently-validated improvements accumulates branch and path-length overhead that outweighs the individual gains (Cycle 0003 composing Cycle 0002's winners; Cycle 0006 composing quota + SEEN_BIT).
- **Adaptive/responsive mechanisms are wasted on a stationary distribution.** Adaptive quota sizing (Cycle 0004) never triggered a single adjustment epoch on the stable Zipfian trace, paying counter-increment and branch overhead on every operation for zero benefit.
- **API-level micro-optimizations run into structural blockers.** Forced inlining pushed hot paths out of I-cache (Cycle 0002); the `entry()` API's borrow-checker constraint forces a pointer release before eviction, erasing its single-probe benefit. Both blockers are inherent rather than fixable.
- **Clone-elision improvements cannot be confirmed on `Copy` keys.** The benchmark uses `Copy` key types where `.clone()` is a register copy; put-path clone optimisations produce no measurable signal and cannot be validated until a heap-owning key type is used.

## Open questions

- **Key-clone elision remains untested under realistic conditions.** A `String`- or `Arc<str>`-keyed benchmark is needed to determine whether the eviction-path refactor produces a genuine put-path win; the hypothesis is still unconfirmed, not falsified.
- **Is there a soft admission filter that avoids SEEN_BIT's over-strictness?** A compact frequency sketch graduating any item with estimated frequency above a low threshold may recover hit-rate selectivity without requiring two sequential hits within a narrow window.
- **Does quota enlargement beyond 87.5% yield further gains or reverse?** The probationary pool has a minimum useful size; the location of the optimum ceiling is unknown.
- **Can arena-allocation or slab-grouping reduce node-access cache-miss cost without any per-op branch?** Structural memory-layout work (node alignment, hot-field packing) has not been fully explored and may open latency headroom without policy complexity.
- **Can the probationary graduation rate be measured directly?** Instrumenting the fraction of items reaching a second hit before eviction would allow data-driven calibration of any future admission threshold rather than inferring it from aggregate hit-rate.

## Per-policy notes

- **LRU (baseline):** Exact ordering consistently outperforms coarser recency signals (CLOCK single-bit, two-hit gate) on workload-a, confirming the workload has enough reuse that precision in eviction decisions pays off.
- **2Q (two-queue):** Hit-rate improvement over LRU is confirmed and stable; the protected-bit-in-prev encoding avoids node growth; 87.5% quota is the settled optimum. Composition with other mechanisms (MRU guard, SEEN_BIT) has produced net regressions in every attempt. The 2Q + quota baseline is the current reference point.
- **CLOCK:** Rejected twice with different implementations. Not worth revisiting unless a mechanism to avoid hand-sweep cache pollution is proposed and motivated by something beyond the current workload.
- **MRU-tail guard (Cycle 0002 winner):** Valid on the workload where it was found; fire rate is too low on workload-a to justify the check cost. Workload-dependent — not a general improvement.
- **SEEN_BIT two-hit admission:** Retired after three cycles. The root cause (insufficient within-window repeat accesses) is structural to the workload, not an implementation artifact.
- **Adaptive quota:** No signal on a stationary distribution; overhead-to-benefit ratio is unfavorable unless workload phase shifts are expected and measurable.
