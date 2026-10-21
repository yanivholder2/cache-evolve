# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants built on cycle-0002 v001 (sampled victims), and each changed one lever. v001 admitted the newcomer on warm ties. v002 widened the victim sample. v003 widened the count-min sketch. Only v003 won. Its gain was clearly larger than any earlier policy tweak, while both contest/sampling levers lost hit-rate.

## Patterns

- The real bottleneck was estimate quality, not the contest rule. Cleaning up collision noise helped. Admitting on ties and searching for victims more deeply both hurt, because they act on estimates that are misleading in two ways: they are inflated by collisions, and they never age, so they reward lifetime counts.
- Strict `>` with keep-victim-on-tie is now tested and should stay. Tie-admission mostly swapped one cold key for another. The get-miss-then-put replay bumps the sketch twice per new key, so a fixed estimate threshold cannot single out one-hit wonders.
- When estimates never age, the lowest estimate marks the youngest resident, not the coldest one. Widening the victim sample therefore evicts recent, still-useful keys. Victim sampling looks exhausted as a lever.
- Part of the "stale-high" inflation blamed on missing aging in earlier cycles was actually collision overcount. Earlier aging results were confounded by this noise.
- For the third cycle running, per-op-time and metadata-size went unscored. v003's larger sketch has a real memory cost that has not been priced yet.

## Open questions

- Now that collision noise is reduced, does very-long-period aging, or doorkeeper-style dedup of the double get/put increment, finally help? Or does lifetime frequency still win on workload-a?
- Is there more to gain from an even wider sketch or a different depth, or has it already plateaued? What is the memory/latency trade-off?

## Continuation

Build on v003, the wider sketch, and treat it as the new baseline for every estimate-related comparison. Next, target estimate meaning: try a doorkeeper or dedup of the get-then-put double count, re-test long-period aging against this baseline, and sweep sketch width/depth while metadata-size is reported. Drop tie-admission and wider victim sampling. Keep insisting on a run that scores per-op-time and metadata-size.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
