# Cycle 0008 Insight

## Cohort outcome

All three variants probed the IRR-based SLRU baseline from cycle 7 — narrowing the probationary scan window, widening the probationary segment, and combining IRR eviction with ghost admission — and every variant regressed. The winner (v003) advances on the back of the smallest regression while confirming that the ghost-admission framework is not complementary to IRR-based eviction.

## Patterns

- The probationary half-scan is load-bearing under IRR for the same reason it was under frequency: IRR values are not uniformly MAX throughout the probationary list. Cold newcomers cluster near the LRU head, but demoted entries with modest IRR values are distributed across the second quarter too, and the scan's selection quality degrades measurably when that range is dropped.
- The 80/20 protected/probationary split is at or near the minimum required to hold this workload's active set. Widening probationary from 20% to 30% triggers excess demotion churn that IRR timestamp preservation cannot absorb — the hot working set is dense enough that a 70% protected segment is genuinely too small, mirroring the cycle-2 regression on the frequency baseline for the same structural reason.
- Ghost admission does not complement IRR eviction. The IRR signal loses the long-term access history that frequency halving preserved across demotion cycles. A node with two closely-spaced accesses far in the past is indistinguishable from a recently-bursted node under pure IRR, so combining ghost-set placement with IRR victim selection yields a worse eviction signal than either frequency-with-ghost or IRR-without-ghost alone.
- The two parameter axes that cycle 7 left open — scan fraction and segment ratio — are now settled on the IRR baseline. Both tested in the same direction as on the frequency baseline: leave them at the values inherited from the prior winning design.

## Open questions

- Does any hybrid signal that blends IRR with a visit count (or with the referenced bit) recover the long-term warmth information that pure IRR discards, without reintroducing the staleness problem of cumulative frequency counts?
- With scan fraction, segment ratio, ghost admission, and the IRR-vs-frequency signal swap all now evaluated, are there structural eviction philosophies not yet tried — such as adaptive segment balancing driven by live hit-rate feedback — that could break out of the current plateau?
- Is the IRR baseline's strength specific to the LIRS-generated trace, or does it transfer to other workloads? Multi-workload evaluation is now the clearest open question.

## Continuation

Next cycle should keep cycle/0007/v003 as the confirmed baseline (IRR timestamps preserved across demotion, 80/20 split, LRU-quarter scan in protected, LRU-half scan in probationary, no ghost set). The probationary scan fraction and segment ratio are both settled — neither narrowing nor widening is productive. Ghost admission is exhausted as a direction on the IRR baseline. The highest-value next steps are: (1) a hybrid signal that combines IRR with a coarse visit count to recover long-term warmth information, or (2) multi-workload evaluation to determine whether the current gains are broadly transferable before further parameter tuning.
