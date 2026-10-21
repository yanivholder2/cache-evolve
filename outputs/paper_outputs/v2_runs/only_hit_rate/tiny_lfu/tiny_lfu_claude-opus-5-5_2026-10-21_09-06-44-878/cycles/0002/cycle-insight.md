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
