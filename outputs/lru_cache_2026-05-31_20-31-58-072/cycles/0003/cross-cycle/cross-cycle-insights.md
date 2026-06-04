## Recurring wins

- Layering complementary mechanisms on top of the champion beats widening a single signal: CLOCK gained ground only once a recency-anchored promotion class and an evicted-key memory (ghost ring) were combined, each closing a distinct gap.
- Recency-encoding per-slot state has consistently outperformed frequency-encoding state on the scored trace; promotion-on-second-touch helps, saturating frequency counters hurt.
- Monotonic-magnitude tuning of a mechanism already on the hot path (e.g. ghost-ring length) is the most reliable way to extract a marginal lift once a structural win is in place — though returns diminish quickly.

## Recurring failures

- Variants whose intended payoff lives off the scored axis (per-op time, metadata size) are invisible to selection and cannot win regardless of structural merit; pure representation repacks and frequency-bit additions both fell into this trap.
- The SLRU rebuild has been attempted twice and remains uninterpretable because parent and child were sampled at disjoint capacities — until matched-capacity sampling is enforced, the SLRU-vs-LRU question stays open.
- Edits to a code path that the current trace/size barely exercises score at exact parity with the parent — pass-2 / Hot-side knobs were inert at the sampled capacity because the Cold sweep never ran out.
- Cohorts have repeatedly sidestepped the fitness-vector bottleneck rather than addressing it, narrowing improvement to "hit-rate on one trace."

## Open questions

- How can the loop reward structural wins (per-op time, metadata size) without a broader fitness vector? This meta-bottleneck has been flagged three cycles running and remains unaddressed.
- Are the dormant Hot-side / pass-2 mechanisms actually dead, or would they bite at smaller capacities or on traces where the Cold pool empties and pass 2 is forced to fire?
- What pass-1 / Cold-side levers remain untried — Cold-admission filtering, ghost-hit promotion targeting, scan-resistance heuristics on the Cold sweep itself?
- Is the single scored trace now the limiting factor? Marginal returns on lirs:multi1 are thinning; trace diversity may be the next bottleneck rather than policy shape.

## Per-policy notes

- LRU baseline: structurally permissive — large representational rewrites preserved invariants and arena-slot-reuse cleanly, so the scaffolding is not the constraint.
- SLRU: twice attempted, twice uninterpretable due to capacity-sampling mismatch; the mechanism itself has never been honestly evaluated. Do not queue again without matched-capacity guarantees.
- CLOCK (plain): baseline structural win, but invisible on hit-rate alone; survived to seed the lineage.
- GCLOCK / frequency-counter CLOCK: regressed on the recency-skewed trace by shielding stale-popular keys against cold-scan inserts; drop until other workloads are scored.
- CLOCK-Pro-lite (Cold/Hot promotion + ghost ring): current champion lineage. Ghost-ring size is the only knob with traction so far, and it is in strong diminishing returns. Hot-side variants (sticky demotion, Hot-ghosts, warm re-admit) all scored at parity on lirs:multi1@512 because pass 2 is dormant there.
