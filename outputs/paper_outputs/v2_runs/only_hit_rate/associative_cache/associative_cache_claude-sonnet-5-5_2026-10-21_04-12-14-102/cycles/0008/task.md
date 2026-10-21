# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0008

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three probes of the small-probation segmented LRU were tried: a fixed 2-slot probation (v001), a recency-stamp compare that lets stale protected entries be evicted (v002), and a lean positional 4-slot split rebuilt on the baseline with no extra state (v003). Only v003 was a Winner; v001 and v002 each lost a little hit rate against their parents.

## Patterns

- The probation-size trend has turned. Going below the slots/16 sizing slightly hurt, so the current size is at or near the optimum and further shrinking is exhausted.
- Eviction-side tweaks did not help. Letting raw recency override probation-first eviction cost hit rate, because the segment boundary already encodes reuse evidence that one last-access stamp does not capture.
- Together with earlier cycles (ghost history, delayed promotion, frequency windows, size tuning), most simple levers on this split have now been tried and found flat or negative. The simple structural split holds almost all the gain.
- v003 again shows that a lean positional split with no ghost, counter or adaptive state improves on plain LRU. Its comparison is against the baseline, not an increment over the current best.
- Per-op-time and metadata-size were unmeasured for the fifth cycle running. v003's "lean" claim (same footprint and speed as baseline) is still untested, and promotion/demotion reordering on hits is still unpriced.

## Open questions

- Does the lean split really keep baseline metadata size and per-op-time, and do the gains survive once those are measured?
- Is there any remaining hit-rate headroom in promotion or protected-segment handling, or is this design at its ceiling for the trace?
- Does the result hold on traces other than the LIRS-style one, or is the tuning specific to it?

## Continuation

Treat the lean 4-slot positional split (v003, with slots/16 probation sizing as the reference) as the base. Drop further probation-size shrinking and recency- or stamp-based victim overrides. The next cycle should first get per-op-time and metadata-size into the fitness vector; after that, try only mechanisms that add no per-entry state, or check robustness on other workloads.


### Cycle 0006

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three probes were tried: doubled associativity as an increment on the best lean split with probation scaling at slots/16 (v001), the same doubling with probation held at an absolute 4 slots (v002), and a policy-preserving speed refactor of the rotate-based split (v003). Only v001 was a Winner, with a very small hit-rate gain. v002 lost marginally, and v003 was a hit-rate tie that cannot be judged.

## Patterns

- Wider buckets do stack on the incumbent, but the increment is tiny. The remaining misses look like capacity or recency-ordering misses, not bucket-conflict misses.
- Proportional probation sizing (slots/16) held up better than an absolute 4 slots on wider buckets. Associativity and probation size are only weakly coupled, and the absolute-scan-length reading is not supported.
- This is the seventh cycle with only hit-rate visible. v003's refactor (early-exit scan, minimal-range rotate, reused miss-scan) is unjudged on its goal. The cost of wider scans in v001 is also unpriced.
- Both v002 and v003 authors could not inspect the code. v003's exact tie confirms policy preservation only if the edit actually took effect.
- The hit-rate levers on this split are essentially exhausted: size, promotion, demotion, ghost state and now associativity all give flat or marginal results.

## Open questions

- Does v003 lower per-op-time, and does v001's wider bucket raise it enough to cancel its small hit-rate gain?
- Is the hit-rate ceiling of this design on the trace reached, so that further gains need an admission or eviction policy of a different kind?
- Do the results hold on traces other than the LIRS-style one?

## Continuation

Stop probing hit-rate knobs in the segmented split (associativity, probation size or sizing rule, promotion, demotion). Treat v001 (wide buckets, slots/16 probation) as the hit-rate base only if per-op-time shows its cost is acceptable. Put per-op-time and metadata-size into the fitness vector first, then judge the v003 rotate refactor against it. After that, try either a different policy family or robustness checks on other workloads.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
