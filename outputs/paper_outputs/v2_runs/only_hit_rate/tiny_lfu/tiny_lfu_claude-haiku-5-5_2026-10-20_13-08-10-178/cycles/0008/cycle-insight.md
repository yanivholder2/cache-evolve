# Cycle 0008 Insight

## Cohort outcome

The cohort tried three changes to how probation victims are chosen or credited: a tie-breaking rule change, a two-candidate victim scan, and a ghost-history frequency bonus. The only Winner, v002, compared the two LRU-tail probation entries and evicted the one with the lower sketch estimate; its hit-rate gain is modest.

## Patterns

- The victim-selection step is where this cohort's signal sits. The Winner changed which probation entry leaves, while the tie rule and ghost bonus only nudged decisions at the margin.
- Tie handling matters. Admitting equal-estimate newcomers against probation victims hurt, because ties were previously broken by queue position and that position was protecting older cold entries the sketch could not separate.
- Ghost-history credit is marginal. The +1 bonus for recently evicted keys barely moved hit-rate, and its ghost ring and per-operation work are unmeasured costs, so it does not earn a place as-is.
- The Winner's attribution rests on the hypothesis and the fitness delta, since its source overlay could not be inspected. Treat the victim rule as the likely cause until confirmed.
- Everything comes from one trace at one cache size, so all differences here are provisional.

## Open questions

- Does the colder-of-two-tail rule hold on other workloads and cache sizes, and does a wider scan window beat a two-entry comparison?
- The victim comparison depends on sketch estimates, so does the Winner's gain depend on the width and conservative-update estimator changes from earlier cycles?
- Is the Winner's gain from the victim rule itself, or from a side effect of how the scan changes queue order?

## Continuation

Next cycle, confirm the probation-victim-by-estimate rule on a second workload or cache size before building on it, and test whether a slightly wider tail scan adds anything. Drop the tie-rule change (`>=` against probation victims), and do not pursue the ghost-history bonus in its current form unless a more selective signal is paired with measured metadata and per-operation cost.
