# Cycle 0007 Insight

## Cohort outcome

All three variants tried to give recency or "warm newcomer" evidence some say against lifetime counts, without adding aging. v001 let a ghost hit (a recently evicted key) win an exact tie. v002 added rare random admission for warm losers. v003 rebalanced SLRU to 50/50. v001 won only because it held the parent's hit-rate. v003 was flat and v002 was a small loss, so this is the third flat cycle in a row.

## Patterns

- Changes at the edges of the decision do nothing when counts are unaged lifetime totals. Exact ties at warm estimates almost never happen, and segment boundaries don't matter when the min-estimate victim sample picks survivors by frequency rank. Ghost tie-breaks and probation length both fail this way.
- Neither the admission contest nor the victim sample ever reaches stale-high residents. The contest only faces probation, and protected keys leave only through demotion. Jitter therefore just churns already-cold probation keys and swaps a higher-count key for a lower-count one.
- Recency evidence, whether ghost history or a longer trial, cannot act unless it moves the comparison by a margin or the counts themselves decay. This contradicts the cycle 0003–0004 verdict that aging is negative for workload-a. That verdict was reached on an earlier design (no SLRU, no dedup at first).
- v001's ghost ring adds metadata. Under hit-rate-only scoring that cost goes unpriced, and the ring is pure overhead if it never changes an outcome.

## Open questions

- Would a margin-based ghost admission (ghost hit plus a small bonus beating the victim) change outcomes, or does it just recreate the tie-admission churn that failed before?
- Is aging negative on the current dedup+SLRU design too, or only on the older ones? Note that cycle 0005's ≤ 1 fast-path shortcuts must be removed if aging returns.
- Can a recency signal be applied to protected-segment demotion instead, where stale-high keys actually live?

## Continuation

Drop exact-tie tweaks, random admission jitter and SLRU ratio sweeps. They are inert or negative. If hit-rate work continues, run at most one decisive test: either periodic halving on the current SLRU+dedup design, to re-check the old anti-aging verdict, or a ghost-hit margin in the contest. Otherwise, treat workload-a as at its ceiling for this family, and spend effort on getting per-op-time and metadata-size scored rather than on more hit-rate tweaks.
