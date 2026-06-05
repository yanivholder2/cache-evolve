# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0002

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants were single-lever reshapes of the baseline DASH tiering,
each targeting hit-rate on a skewed trace without touching capacity. The
winner (v002) repartitioned fixed budget toward the promotion-protected
normal tier; the two stash-admission tweaks (v001 back-insertion, v003
frequency gate) also moved hit-rate favorably but did not top it.

## Patterns

- Every variant converged on the same lever family: protect the warm working
  set in the normal tier by reducing how easily transient keys disturb it,
  whether via capacity reallocation, insertion position, or a promotion gate.
- The baseline appears to over-provision the cold stash admission area; both
  shrinking it (v002) and making it more selective (v001, v003) helped, which
  points to a single underlying weakness rather than three independent ones.
- All wins were budget-conserving and invariant-preserving — no dimension
  regressed in any variant, so the cohort moved along one axis (hit-rate)
  with no observed trade-off cost.

## Open questions

- Are these levers additive or redundant? They each attack the same
  stash-vs-normal imbalance, so stacking v002 with v001/v003 may yield
  diminishing or conflicting returns.
- Both v002's split ratio and v003's promote threshold are unswept knobs; the
  reported wins are single points, not optima.
- Results rest on one skewed trace; the stash-protection bias may not
  generalize to less skewed or churn-heavy workloads.

## Continuation

Pursue the stash-vs-normal balance as the dominant theme — the winning move
was reallocating budget to the protected tier. Next cycle should sweep the
v002 split ratio and v003 threshold for optima, and test whether combining
the split shift with a selective stash gate compounds or cancels. Drop pure
re-runs of single-lever baselines; the cohort has established the direction.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
