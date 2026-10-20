# Cycle 0007 Insight

## Cohort outcome

The cohort tried three changes: halving the protected-segment share so the demotion branch fires more often (v001), breaking sketch-estimate ties in favour of the candidate at admission (v002), and choosing the counter block from the key hash and the column from the per-row seed (v003). All three are labelled Winners, but the variant evidence does not support a clear gain: none raised hit rate on workload-a at size 2048, and each reports a per-op-time or hit-rate regression.

## Patterns

- Hit rate did not rise in any variant. v001 and v002 both dipped slightly, and v003 recovered only a trivial amount of the parent's loss. Admission tie-breaking and protected-capacity changes are not where this trace loses hit rate.
- Protected-segment capacity matters more than protected-tail turnover. Halving the protected share made the demotion branch reachable, and that cost hit rate and per-op time. This partly answers the Cycle 0006 question: the branch can fire, and firing it more often did not pay off.
- Per-op time regressed in every variant, at both get and put. The causes are inferred (extra list moves, extra per-row lookup work, extra evictions on ties), not measured from source.
- The speed gain from Cycle 0006's cache-line grouping depended on the simpler column mapping. Taking the column from the per-row seed did not restore collision spread and gave up the get-path benefit, so the speed-versus-collision trade-off is still unresolved.
- Metadata is essentially flat. The one size movement, a slightly smaller TinyLfuCache in v003, is unexplained.
- Every result comes from one run on one trace (workload-a), and no variant's overlay source was inspected.

## Open questions

- Is there a protected capacity that still exercises the demotion path without losing hit rate, or does any reduction cost hit rate on this trace?
- Does the admission tie case matter on a second trace, or is it inert on workload-a?
- Can the cache-line grouping keep its get-path gain without a per-row column lookup?
- Do the put tail regressions in v001 and v002 survive repeat runs?
- What reduced the TinyLfuCache size in v003?

## Continuation

Drop the tie-break change (v002) and the block-plus-seeded-column split (v003) as directions on this workload: neither moved hit rate, and both added per-op cost. Keep the protected-capacity lever (v001) as the one open thread, but test whether the demotion path can run at a capacity that holds hit rate before exploring protected-tail turnover again. Revisit cache-line grouping only if a collision-preserving block selection can be found that does not add a per-row column lookup.
