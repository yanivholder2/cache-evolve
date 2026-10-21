# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three variants were tried: a halved sketch aging threshold, a stricter admission margin (candidate must beat the victim by more than 1), and a CLOCK + conservative-update sketch gate replacing strict LRU. Only the stricter admission margin (v002) won, with a marginal hit-rate gain. The shorter aging window lost slightly. The CLOCK variant reported a hit-rate gain over strict LRU, but it did not win.

## Patterns

- Aging-window tuning is now bracketed from both sides. Longer (cycle 0007) and shorter (this cycle) both lost slightly, so the window sits at a flat optimum.
- The admission-margin win contradicts the earlier "admission gate is saturated" conclusion only weakly. The gain is small and may be noise. It suggests a stricter gate trades a few lost second-touch keys for less scan pollution, and counting rejected misses keeps that trade safe.
- CLOCK + sketch again shows a hit-rate edge over strict LRU without beating the segmented line. This is the third cycle with that pattern (0003, 0006, 0009).
- Metadata-size and per-op-time were again missing from the fitness vectors, now for nine cycles. The claimed cheaper CLOCK hits and the sketch costs remain unverified.
- Variant authors could not read the overlay source, so their interpretations rest on hypothesis text and fitness vectors only. All differences are marginal, on one trace at one cache size.

## Open questions

- Does the stricter margin hold on traces with more second-touch reuse or less scan pressure?
- Does CLOCK's cheaper hit path offset its miss-time sweep and sketch updates? That cannot be known without timing data.
- Is the missing cost measurement a harness fault?

## Continuation

Stop tuning the aging window and the admission margin; both are at a plateau on this trace. Treat v002's margin as a minor, optional tweak on the incumbent. Fix metadata-size and per-op-time measurement and add a second trace or cache size before generating more hypotheses, since the loop is selecting on marginal hit-rate edges alone. Then try a different axis, such as a new eviction signal (size or cost awareness), or a head-to-head cost-aware comparison of CLOCK + sketch against the segmented incumbent.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
