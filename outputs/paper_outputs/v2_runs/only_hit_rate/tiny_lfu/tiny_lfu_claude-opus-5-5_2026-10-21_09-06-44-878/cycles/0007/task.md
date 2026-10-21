# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants built on the cycle-0003 wide sketch and each changed one thing about what the estimates mean. v001 deduplicated the get-miss-then-put double count. v002 added conservative update. v003 added very-long-period halving aging. Only v001 won, by a small margin. The other two were flat, within noise.

## Patterns

- Making estimates more accurate has stopped paying off. On the 16×-wide sketch, conservative update made no difference, because collision overcount is already gone from the minimum row. Width and conservative update fix the same error, so their gains don't add up.
- Aging is now settled as negative for workload-a. It still didn't help on a clean sketch at a very long period. This confirms that the earlier "stale-high" blocking came from collisions, not from genuinely old popularity. Lifetime frequency is the right signal for this trace.
- Making counts honest helped, but only a little. With dedup, sketch counts match true access counts, so keys that keep missing no longer outrank residents that collect counts through hits. But a first-time newcomer still ties a resident that was accessed once, and the newcomer loses that tie.
- The sketch side (width, update rule, aging, dedup) now looks used up as a hit-rate lever. Over four cycles, the only large gain came from removing collision noise.
- This is the fourth cycle in a row where per-op-time and metadata-size went unscored. v001 should lower per-op cost (one hash per get, and fewer increments), but that has not been measured.

## Open questions

- Is the remaining hit-rate in the resident structure instead? For example, a small window or doorkeeper in front of the main region, or an SLRU probation/protected split, now that estimates are clean and honest. These were only tested earlier on the noisy sketch.
- How much can the sketch shrink before hit-rate drops, now that conservative update can make up for collisions at a narrower width? This matters once metadata-size is priced.

## Continuation

Build on v001 (wide sketch with dedup) as the new baseline. Drop further work on sketch accuracy, all aging, tie-admission and wider victim sampling. Next, re-test structural levers on the clean estimates, such as an SLRU-style protected/probation split or a small fixed window. If metadata-size is finally scored, run a narrower-sketch-plus-conservative-update test to recover memory. Keep demanding per-op-time and metadata-size reporting.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle-0004 v001 (wide sketch with dedup). Only one of them changed policy. v001 split the residents into an SLRU probation/protected layout and sampled victims only from probation. It won, but by a marginal hit-rate gain. v002 (a narrower sketch with u16 counters and conservative update) and v003 (a semantics-preserving per-op speedup) were built to move metadata-size and per-op-time. Both matched the parent's hit-rate exactly. Because only hit-rate is scored, the evaluation could not see what they were built to improve.

## Patterns

- Structural levers on clean estimates are also close to exhausted. The 8-way min-estimate victim sample was already acting as a soft protected segment, so an explicit SLRU split mostly duplicates it. The split also adds per-hit list work, which is unpriced.
- The sketch is over-provisioned. Half the width with half-size counters plus conservative update left every admission and victim decision unchanged. This confirms that width and conservative update substitute for each other, and that lifetime counts stay far below u16 saturation.
- Hit-rate on workload-a now looks stuck on a plateau around the parent. Three different kinds of change landed on or barely above it. The remaining differences between candidates are in cost axes that are not measured.
- The evaluator has become the bottleneck. This is the fifth cycle with per-op-time and metadata-size unscored. Two of the three variants this cycle are wins that cannot be verified, and the Winners selection cannot reward them.
- New coupling: v003's shortcuts (early stop at estimate ≤ 1, early rejection of newcomers at ≤ 1) are correct only while there is no aging. Any future change that adds decay or reset must re-derive them.

## Open questions

- After per-op-time and metadata-size are scored, which lineage dominates? Possible answers include SLRU, the narrow CU sketch, the hash-cached fast path, or a combination of all three, which should be compatible.
- How far can the sketch shrink (narrower width, or smaller counters) before hit-rate moves?
- Is there any hit-rate left on workload-a, for example from a separate admission window in front of SLRU or a different probation ratio, or has the trace reached its ceiling for TinyLFU-style policies?

## Continuation

Treat hit-rate on workload-a as plateaued and stop spending variants on small policy tweaks. The top priority is getting per-op-time and metadata-size into the fitness vector. Then combine v002's narrow CU sketch and v003's hash-cached fast path with v001's SLRU, and price SLRU's per-hit cost. Probe a narrower sketch only once metadata-size is scored. Any aging proposal must remove v003's ≤ 1 shortcuts.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants built on the cycle-0005 SLRU winner. v001 added a 1% W-TinyLFU admission window. v002 folded in the narrow u16 conservative-update sketch. v003 picked the protected entry to demote by frequency instead of by LRU. v003 won only because it held the parent's hit-rate unchanged. v001 and v002 both landed slightly below the parent, so this cycle produced no real gain.

## Patterns

- The SLRU design is over-defended. Probation's second-chance MRU placement and the frequency-aware victim scan already protect hot keys. Changing which entry gets demoted, or adding a small window in front, rearranges entries without changing which keys get evicted.
- The window lever is now tested on clean estimates and is negative. Few keys on workload-a come back within a short recency trial, and the slots the window takes from the main region cost more hits than the window recovers. Losing ties as a first-time newcomer is not where the remaining misses are.
- In SLRU, the narrow sketch is no longer free. Probation-only sampling compares the newcomer against a few low-count entries, so small amounts of leftover collision error now flip some decisions. Width 8× looks like the floor for this design. In the non-SLRU parent it was exactly neutral.
- This is the sixth cycle with per-op-time and metadata-size unscored. H02 assumed metadata-size was on the leaderboard, but it wasn't. The cohort's only memory-saving variant therefore counted as a small loss. v003 also adds unpriced per-promotion estimate cost.
- Hit-rate on workload-a has now held flat for two cycles across policy, structure and sketch levers.

## Open questions

- Could a recency tie-break between equal estimates let the sketch go below 8× in SLRU without flipping decisions?
- Is anything left on workload-a beyond changing what enters probation? Is the plateau simply this trace's ceiling for TinyLFU-family policies?

## Continuation

Stop spending variants on SLRU-internal hit-rate tweaks. Drop small windows, demotion policy, victim sampling and aging. Before proposing any memory or latency variant, confirm that metadata-size and per-op-time are actually being scored. If they are not, such variants can only tie or lose. If they are scored, re-run the narrow sketch on SLRU with a recency tie-break between equal estimates, and price v003's extra estimates against the plain-LRU-demotion parent.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
