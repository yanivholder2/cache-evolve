## Recurring wins

- Structural separation of one-touch from re-referenced entries (a probationary/protected split in each bucket) is the one large, repeatable gain. It was confirmed from several independent routes (cycles 2-8).
- A lean positional split with no extra per-entry state (no ghost, counters or adaptive targets) captures nearly all of that gain. It has been rebuilt on the baseline several times and each time beat plain LRU.
- A small probation segment (about slots/16) beats larger ones. The size trend turned in cycle 5, so the current size is at or near the optimum.
- Wider associativity stacks on the split, but with sharply diminishing increments: a clear gain at first, then marginal, then marginal again. Proportional probation sizing held up better than an absolute size.
- Inserting new keys just above the probation boundary keeps scan protection without starving them of residency.

## Recurring failures

- Weakening hit-path promotion in plain LRU buckets (transposition, midpoint, half-promotion) lost hit rate.
- Delaying promotion (two-hit rule) or victim-end insertion (LIP-style) lost hit rate. The trace has short reuse distances, so new keys need residency to earn a first re-reference.
- Ghost history and direct ghost-hit promotion added nothing over the split. A ghost hit is weak reuse evidence and can displace entries that had earned protection.
- Small-window frequency victim choice and recency-stamp overrides of probation-first eviction were flat or negative. The segment boundary already encodes the reuse evidence.
- Demotion placement is a neutral lever (exact ties).
- Multi-bucket or two-choice placement fails the slot-legality invariant. It is incompatible with the gate as written.
- Many "wins" are ties, or are measured against the baseline instead of the current parent. They are not real increments.

## Open questions

- Per-op-time and metadata-size have been missing from the fitness vector in all 8 cycles. Is this fixable? No cost-side claim has ever been confirmed: the wide-bucket scan cost, the rotate refactors and the skip-move rule are all unjudged.
- Do the exact-tie results reflect real neutrality, or edits that never changed behaviour? Authors could not read the overlay code.
- Is the hit-rate ceiling of this design on the trace reached? Would a different policy family (such as frequency-aware admission) be needed?
- Do the results hold on traces other than the LIRS-style one?
- Is a mid-probation insertion point better than either extreme?

## Per-policy notes

- Plain move-to-back LRU: the baseline. Every promotion-weakening tweak lost to it.
- Segmented LRU (probation/protected, first-hit promotion, small probation of about slots/16, boundary insertion): the current incumbent. Its hit-rate levers (size, promotion, demotion, insertion, ghost state, associativity) are nearly exhausted.
- Ghost-ring or adaptive 2Q variants: unnecessary state, with no gain over the lean split.
- Frequency-based victim choice: no effect at small windows.
- Wide buckets (2x-4x associativity): a small hit-rate gain with unpriced scan cost. Accept this only if per-op-time shows the cost is tolerable.
- Direction: stop tuning hit-rate knobs. First measure speed and size, then try a different policy family or other workloads.
