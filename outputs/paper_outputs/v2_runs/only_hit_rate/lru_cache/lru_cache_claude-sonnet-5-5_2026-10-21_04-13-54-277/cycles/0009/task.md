# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0009

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three variants were tried from the doorkeeper-free SLRU + sketch incumbent (cycle 0007 v003): an ARC-style adaptive protected share with a ghost list, a small admission window in front of probation, and conservative-update 4-bit packed sketch counters. Only the sketch-precision variant (v003) won, with a marginal hit-rate gain. The adaptive share was noise-level, and the admission window lost slightly.

## Patterns

- Every change was within noise on hit-rate, again on one trace at one cache size. The incumbent looks close to a local optimum on this workload.
- Adaptive segment sizing found almost no trace-phase variance to exploit. The fixed protected share was already near its best, so the ghost list and demotion bookkeeping add complexity for essentially no return.
- Deferring admission through a tiny window did not recover keys that entry-time gating wrongly rejected. The window is too short for bursty keys to be reused before they face the gate. Entry-time gating is close to sufficient here.
- Sketch precision is a minor lever. Cleaner counts helped only slightly. The intended payoff, a smaller sketch, was not measured. Metadata-size and per-op-time have been missing for eight cycles, so the winner may have won only on a tiny hit-rate edge.
- Variant authors in this cycle could not read the overlay source, so these interpretations rest on hypothesis text and fitness vectors only.

## Open questions

- Does the 4-bit packed sketch really halve metadata without hurting hit-rate or per-op-time? That is unknowable until the cost metrics are measured.
- Would a larger window, or a different workload, make deferred admission or adaptive sizing matter?
- Is the missing cost measurement a harness fault?

## Continuation

Stop tuning segment sizing, admission timing and sketch update rules; all are plateaued on this trace. Treat cycle 0008 v003 (compact conservative sketch) as the new incumbent only if the cost metrics can be verified. Before more hypotheses, fix metadata-size and per-op-time measurement and add a second trace or cache size. Then try a genuinely different axis, such as a different eviction signal (recency plus size or cost awareness), or test existing ideas on a scan-heavy or long-reuse trace.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
