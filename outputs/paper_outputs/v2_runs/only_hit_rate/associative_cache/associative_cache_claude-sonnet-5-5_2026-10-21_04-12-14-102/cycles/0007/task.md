# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0007

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three probation-segment variants were tried: shrinking probation further (v001), a stricter two-hit promotion rule (v002), and a minimal fixed probation share built directly on the baseline geometry (v003). The two Winners were the small-probation variants (v001, v003). The stricter promotion rule lost hit rate.

## Patterns

- Two independent routes both ended at a small probation segment (about 4 slots per bucket) and both won. The trend toward smaller probation has not clearly turned, but the gain from v001 is marginal and looks to be flattening.
- v003 reports a large gain over plain LRU without the cycle 1-3 additions (ghost rings, counters, adaptive state). This suggests the simple split plus small probation captures most of the benefit, and the extra bookkeeping from earlier cycles may be unnecessary.
- Delaying promotion hurt (v002). With a small probation segment, entries cannot wait for a second hit without being evicted. Together with cycle 0003's ghost result, first-hit promotion looks hard to beat here.
- Per-op-time and metadata-size were unmeasured for the fourth cycle running. Every claimed win is hit-rate only, and the cost of segment reordering and demotion on `get` is still unpriced.
- v003's comparison is against the baseline, not the current parent, so its size is not an increment over v001.

## Open questions

- Where is the actual minimum of probation size? Is going below 4 slots (for example 2, or 1) still better, or does scan resistance break?
- Does v003's simple split match the full accumulated stack, and can the older extra state be removed?
- Do the hit-rate gains survive once per-op-time and metadata-size are measured?
- Would a larger probation segment make a two-hit rule viable, or is first-hit promotion simply better?

## Continuation

Probe probation sizes just below the current point (and one above, as a check) to find the turn, and consider simplifying by stripping unneeded earlier state, using v003's lean split as a reference. Drop two-hit or other delayed-promotion rules in a small probation segment. Fix the evaluation so per-op-time and metadata-size appear in the fitness vector before adding any new bookkeeping.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
