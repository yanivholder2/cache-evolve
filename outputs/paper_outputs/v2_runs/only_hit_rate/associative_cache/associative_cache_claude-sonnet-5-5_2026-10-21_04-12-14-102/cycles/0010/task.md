# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three probes were tried: LIP-style insertion at the probation eviction end (v001), a further doubling of associativity to 4x baseline at fixed capacity with slots/16 probation (v002), and a boundary-insertion lean split with a skip-move rule for hits already in the recent quarter (v003). Only v002 was a Winner, with a marginal hit-rate gain. v001 lost hit-rate against its parent, and v003 improved on plain LRU but did not beat the incumbent.

## Patterns

- Insertion position is the one lever that moved things this cycle, in both directions. Inserting at the victim end starved new keys of the residency needed to earn a first re-reference. Inserting just above the probation boundary kept scan protection without that starvation. The trace's short reuse distances punish aggressive scan resistance.
- The associativity increment shrank again. The first doubling gave a clear gain, the second a marginal one, so the remaining misses look capacity- or recency-driven, not conflict-driven.
- v003 repeats the earlier finding that a lean positional split with no extra per-entry state beats plain LRU. Its comparison is against the baseline, not against the current best.
- The hit-rate levers on this split are now almost fully worked: size, promotion, demotion, ghost state, associativity and insertion point all give flat, marginal or negative results.
- For the eighth cycle running, only hit-rate is visible. The cost of wider scans in v002 and of positional checks in v003 is unpriced. Authors could not read the overlay code, so their explanations are inferred.

## Open questions

- Does the cost of 4x-wide bucket scans cancel the marginal v002 gain once per-op-time is measured?
- Is a mid-probation insertion point between MRU and the boundary better than either extreme, or is the parent's choice already near optimal?
- Is further gain only possible from a different policy family, such as frequency-aware admission, or does it need other workloads to show a difference?

## Continuation

Stop probing hit-rate knobs in the segmented split: associativity beyond the current width, LIP or victim-end insertion, probation size, promotion and demotion. Get per-op-time and metadata-size into the fitness vector first, then judge v002's width and v003's skip-move rule against them. After that, try a different policy family or robustness checks on other traces.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three probes were tried on the 4x-wide split: mid-probation insertion (v001), skipping the protected-MRU rotate for hits already in the top quarter (v002), and a larger slots/8 probation fraction on wide buckets (v003). v002 was the only Winner, and only as an exact hit-rate tie with its parent. v001 lost slightly, and v003 beat the baseline but was not compared against the incumbent.

## Patterns

- Insertion position is now negative at both extremes. Victim-end insertion (cycle 0008) and mid-probation insertion (this cycle) both cost hits, because new keys need residency to earn a second reference. Insertion point is not a useful knob.
- The Winner is a speed-motivated refactor with identical hit-rate, so it wins by default. Its goal, lower per-op-time, is again unmeasured.
- v003 repeats the pattern of a lean positional split beating the baseline. It does not show that slots/8 beats the incumbent's slots/16, so the probation fraction question is still open.
- Per-op-time and metadata-size are missing for the ninth cycle running. The cost of 4x-wide scans, segment shuffles and skip-move checks has never been priced.
- Authors again could not read the overlay code, so their explanations rest on the hypothesis text and are inferred. An exact tie may also mean the edit had no effect.

## Open questions

- Does the skip-move rule lower per-op-time enough to justify keeping it?
- Does the 4x width, with its marginal hit-rate gain, survive pricing in time?
- Is the hit-rate ceiling for this design on this trace reached, so that further gains need a different policy family?
- Do the results hold on other traces?

## Continuation

Stop probing hit-rate knobs in the segmented split: insertion point (victim-end or mid-probation), probation fraction, associativity, promotion and demotion. Get per-op-time and metadata-size into the fitness vector before another cycle, so v002's skip-move and the wide buckets can be judged. After that, either try a different policy family such as frequency-aware admission, or run robustness checks on other traces.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
