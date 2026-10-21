# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
