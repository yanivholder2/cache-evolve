# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0008

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three variants targeted probation on the 95%-protected segmented LRU: a TinyLFU-style sketch admission gate, a change to where demoted protected nodes land in probation, and a frequency-aware probation eviction scan. Only the admission gate (v001) won, with a marginal hit-rate gain. The other two matched their parent exactly.

## Patterns

- Probation-side tweaks are nearly inert at this protected share. Demotions are rare, so where demoted keys land in probation changes nothing.
- The frequency-scan variant was dead by construction. Frequency was reset or never persisted in the segment being scanned, so every candidate tied and the scan fell back to plain LRU order.
- Admission filtering gave a small gain, but segmentation already keeps one-hit scan keys in the small probation segment. The sketch's extra work (hashing, counters, periodic decay) buys little. Counting rejected misses lets them be admitted on retry, and the gate can reject new hot keys.
- Hit-rate gains remain marginal and rest on one scan-heavy trace at one cache size.
- Metadata-size and per-op-time were not used to judge anything this cycle, and the cost of the sketch and the extra freq byte per node is unverified.

## Open questions

- Is the sketch gate worth its time and memory cost, if per-op-time and metadata-size could be measured?
- Does admission filtering matter more on traces with long-distance reuse or heavier scan pressure?
- Would frequency help if it persisted across demotion, or covered protected entries?
- Is the measurement gap on metadata-size and per-op-time (now five cycles) a harness fault?

## Continuation

Stop varying probation placement and probation-only eviction scans; the segmented design is saturated on this trace. Keep the admission gate only if cost can be measured, and prefer cheaper variants such as not counting rejected misses or a tiny doorkeeper. Before more hypotheses, fix the cost measurement and add a second trace or cache size. Then try ideas that change a different axis, such as adaptive sizing, CLOCK hybrids, or frequency that persists across segments.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three variants worked on the sketch admission gate or its CLOCK alternative: no longer counting rejected misses, a one-bit doorkeeper in front of the sketch, and a CLOCK + sketch-gate rewrite of the strict LRU. Only the doorkeeper (v002) won, with a very small hit-rate gain. Not counting rejected misses lost a little hit-rate.

## Patterns

- Admission-gate tuning is saturated. The segmented LRU at 95% protected share already filters most scan keys, so cleaner frequency estimates leave little to gain.
- Counting rejected misses helps. It lets recurring keys build frequency across retries and earn admission. Removing it hurt.
- The doorkeeper's benefit (less scan pollution in the counters) is mostly offset by its costs. A new hot key estimates only 1 and loses the gate more often, and each halving forgets recent first sightings.
- CLOCK plus a sketch gate reports a hit-rate gain over strict LRU, but it did not win. The comparison base is unclear. This is the same "CLOCK is incremental" pattern seen in cycle 0003.
- Metadata-size and per-op-time were again missing from the fitness vectors, now for six cycles. The cost of the sketch, doorkeeper and CLOCK hand sweep is unverified.
- All gains are marginal and rest on one trace at one cache size.

## Open questions

- Is the doorkeeper's tiny gain worth its added state and logic, given that costs cannot be measured?
- Would the CLOCK + sketch design beat the current best segmented line if compared head to head, and on what cost profile?
- Does admission filtering matter on traces with long-distance reuse or heavier scan pressure?
- Is the missing cost measurement a harness fault?

## Continuation

Stop tuning the admission gate: the sketch update rule, the doorkeeper and the threshold are all at a plateau on this trace. Before more hypotheses, fix metadata-size and per-op-time measurement and add a second trace or cache size, since the loop is selecting on hit-rate alone with only marginal differences. Then pursue a different axis, such as adaptive segment sizing, frequency that persists across segments, or a direct cost-aware comparison of the CLOCK + sketch design against the segmented winner.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three variants were tried: a frequency-aware victim scan over the oldest probation entries, a much longer sketch aging window with a decoupled doorkeeper reset, and a simplified rebuild of the segmented LRU with a sketch gate and no doorkeeper, starting from baseline. Only the simplified rebuild (v003) won. The victim scan was marginal at best, and the longer aging window lost a little hit-rate.

## Patterns

- Hit-rate was again the only measured dimension. The winner's claimed savings in per-op-time and metadata-size are unverified, so it is unclear whether it won on hit-rate or only on a more compact design. This is the seventh cycle with the cost metrics missing.
- Slower sketch decay hurt. Stale high-count keys block newer candidates at admission, so the aging window should stay near its current length. The test also changed the doorkeeper reset, so the two effects cannot be separated.
- Frequency-aware victim selection now has a real signal because frequency persists across segments. The gain is small, which suggests the probation LRU head was already close to the best victim.
- The simplest design (SLRU plus a sketch gate that counts rejected misses, no doorkeeper) was competitive. This is consistent with earlier cycles, where the doorkeeper's gain was small and offset by its costs.
- All differences are marginal and rest on one trace at one cache size.

## Open questions

- Does the doorkeeper-free design hold hit-rate while being cheaper? That cannot be known until per-op-time and metadata-size are measured.
- Does the victim scan add per-insert cost that outweighs its small gain?
- Would a smaller aging-window step, or testing the window and the doorkeeper reset separately, show anything?
- Is the missing cost measurement a harness fault?

## Continuation

Stop lengthening the aging window and stop widening the victim scan; both are at a plateau on this trace. Treat the doorkeeper-free SLRU plus sketch (v003) as the simplicity baseline. Fix the metadata-size and per-op-time measurement and add a second trace or cache size before any more hypotheses. After that, try a different axis, such as adaptive segment sizing, or a cost-aware comparison of v003 against the doorkeeper variants.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
