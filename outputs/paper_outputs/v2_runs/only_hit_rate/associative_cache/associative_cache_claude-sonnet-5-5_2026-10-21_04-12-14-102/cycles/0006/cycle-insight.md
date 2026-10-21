# Cycle 0006 Insight

## Cohort outcome

Three probes were tried: demoted-entry placement at the probation LRU end (v001), a rotate-based rewrite of promotion/demotion aimed at per-op-time (v002), and doubled associativity combined with the lean 4-slot split on the baseline (v003). v001 and v002 won only by tying their parents on hit-rate. v003 improved clearly over baseline, but that is a gain against the baseline and not over the current best.

## Patterns

- Both parent-derived variants were exactly neutral on hit-rate. Demotion placement is not a lever, which fits the earlier finding that most simple levers on this split are flat. The two Winners are ties, not improvements.
- v002 was a policy-preserving refactor, so hit-rate cannot judge it. Its goal was per-op-time, and that has still not been measured. The two Winners say nothing about speed.
- v003 suggests wider buckets and the small probation segment stack. This is an unpriced claim: longer bucket scans and segment shuffles may cost latency.
- Per-op-time and metadata-size were unmeasured for the sixth cycle running, so no cost-side claim has ever been confirmed or refuted.
- Variant authors could not read the overlay code, so a tie that exact may also mean the edit never changed behaviour.

## Open questions

- Does the rotate rewrite lower per-op-time, and do the wider buckets raise it?
- Is v001's exact tie real neutrality, or a demotion path that rarely fires or an edit with no effect?
- Does the associativity gain over the incumbents hold, or is it only a gain over the baseline?
- Do the results hold on traces other than the LIRS-style one?

## Continuation

Stop probing hit-rate levers inside the segmented split (demotion placement, probation size, promotion rules, ghost state); they are exhausted. Make per-op-time and metadata-size measurable first, since v002 and v003 can't be judged without them. Then take wider associativity from v003 and test it as an increment on the current best parent, or check robustness on other workloads.
