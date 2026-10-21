# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0003

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants started from the baseline, and each tried one separate idea: a W-TinyLFU window/segmented layout (v001), sketch aging plus conservative update (v002), and a single fast hash with a flat sketch (v003). The only Winner was v003, a speed-oriented change that is neutral on policy. Its hit-rate gain was small and probably comes from a different pattern of hash collisions, not from a better policy. Neither of the two policy changes beat the baseline in a way that mattered.

## Patterns

- Changing the resident structure (v001) did almost nothing at size 2048. The 1% window holds only about 20 slots, so admission is still decided by the same frequency contest that never ages.
- Removing frequency history (v002) hurt hit-rate. On workload-a, the baseline's long-run counts were worth more than the freshness that decay added. When counts are low and coarse, the strict-`>` contest mostly ends in a tie, and the victim stays.
- Every variant's evaluation reported hit-rate only. Per-op-time and metadata-size were never measured, so the dimension that H03 actually targeted (lower get latency) remains unverified.
- Both policy variants point to the admission contest as the real bottleneck: how estimates are compared and how ties are broken. List layout is not the bottleneck.

## Open questions

- Does aging fail because the sample size (10×capacity) is too small, or because of conservative update? v002 changed both at once.
- Would a larger window, or an adaptive one (hill-climbing), let v001's layout pay off?
- Does v003 actually lower get/put latency? This needs a benchmark run that reports per-op-time.

## Continuation

Build on v003, which keeps the fast hashing. First get a run that reports per-op-time and metadata-size, so the speedup can be confirmed. For hit-rate, aim at the admission contest. Test aging alone, without conservative update, at a much larger sample size, and/or try a tie-breaking or doorkeeper rule. Also try pairing a larger or adaptive window with that aging. Drop pure restructuring of the segments at a 1% window, and drop aggressive halving at 10×capacity.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants kept the same contest and changed one admission-adjacent lever each. v001 used frequency-sampled victim selection among the oldest LRU entries. v002 added long-period halving aging with no conservative update. v003 made the W-TinyLFU window adaptive with a hill-climber. Only v001 won, and its hit-rate gain was marginal. Both other levers lost hit-rate on workload-a.

## Patterns

- Freshness keeps losing on workload-a. Aging alone, without conservative update and over a longer period, still hurt. It hurt less than the cycle-0001 combined change, so most of that earlier damage came from conservative update and/or the short period.
- Strict `>` with keep-victim-on-tie runs through every result. Halving creates ties that newcomers lose. Victim sampling helps only a little, because the oldest entries are uniformly cold or still carry stale-high estimates. Tie handling in the contest has now been named as the bottleneck in two cycles and has never been tested directly.
- Restructuring and adaptation don't pay off on this trace. A naive climber pushed the window toward a larger, LRU-like region, which this LIRS-style trace punishes. That contradicts cycle 0001's hunch that the window was too small.
- Per-op-time and metadata-size are still unmeasured for the second cycle running. v001 adds sketch estimates on miss-puts and v003 adds per-get bookkeeping, so neither latency cost has been checked.

## Open questions

- Does a `>=` contest, or one with a tie-breaker (such as a doorkeeper or a random admit on ties), beat strict `>`, both alone and combined with v001's sampling?
- Would a larger sample, or one placed somewhere else, make victim sampling matter, or is the head region simply cold?
- Would a warmed-up, damped climber with small steps (and a contest on shrink) recover anything, or is a fixed small window simply right for this trace?

## Continuation

Build on v001 and target the contest directly: test `>=` or tie-breaking admission first, then try that with a wider or alternatively placed victim sample. Drop sketch aging and naive window hill-climbing on workload-a. Revisit aging only as a very long period paired with a tie-aware contest. Insist on a run that reports per-op-time and metadata-size, so v001's extra estimates and v003-lineage hashing are actually priced.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
