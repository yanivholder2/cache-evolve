# Cycle 0008 Insight

## Cohort outcome

All three variants probed the remaining sketch and admission axes: sketch depth, periodic count aging, and direct-to-protected admission for sketch-hot items. None produced a gain — the winner (v001) held flat while the other two regressed — making this the first cycle to yield zero advancement.

## Patterns

- **Sketch depth is saturated at the current width.** With 8× width already pushing per-row collision probability to a low floor, additional hash witnesses provide no further discriminating power; the 4-row minimum estimator already returns accurate estimates on this workload. The depth axis is closed.
- **Count aging actively hurts at this timescale.** Even at a very slow halving cadence the accumulated frequency signal erased by each reset outweighs the stale-count noise it removes, indicating the workload does not produce meaningful count inflation at this sketch scale. The aging/reset axis is closed.
- **Probationary is a load-bearing timing gate, not mere overhead.** Bypassing it for sketch-hot admits causes immediate protected-segment churn: direct inserts trigger premature demotions before demand is reconfirmed by a real access. The insight reinforces that the current admission path is correct in structure, not just in tuning.
- **The sketch layer is fully exhausted.** Across cycles 0006–0008 the cohort has closed conservative update, width, depth, hash-seed diversity, and aging. All sketch axes are now confirmed closed.

## Open questions

- Do the policy knobs closed in cycle 0005 (PROMO_THRESHOLD, victim-scan window, SLRU split) behave differently now that the underlying frequency estimates are substantially sharper? The earlier closures were reached with a noisier sketch baseline.
- Is the remaining hit-rate gap attributable to irreducible workload entropy (unpredictable cold items), or does a structural policy change still have room to move?

## Continuation

The sketch is fully tuned and should be treated as fixed. The most promising next probe is to revisit the policy axes closed in cycle 0005 under the now-improved sketch signal: PROMO_THRESHOLD gating and the k=4 victim-scan window are the top candidates, since their earlier conclusions were reached with higher collision noise. If both remain flat under the sharper signal, the design has reached its ceiling for this workload and trace, and further investment in tuning is unlikely to pay off.
