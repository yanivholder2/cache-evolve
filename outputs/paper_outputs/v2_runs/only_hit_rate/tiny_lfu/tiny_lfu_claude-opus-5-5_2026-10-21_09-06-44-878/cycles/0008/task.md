# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0008

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants tried to give recency or "warm newcomer" evidence some say against lifetime counts, without adding aging. v001 let a ghost hit (a recently evicted key) win an exact tie. v002 added rare random admission for warm losers. v003 rebalanced SLRU to 50/50. v001 won only because it held the parent's hit-rate. v003 was flat and v002 was a small loss, so this is the third flat cycle in a row.

## Patterns

- Changes at the edges of the decision do nothing when counts are unaged lifetime totals. Exact ties at warm estimates almost never happen, and segment boundaries don't matter when the min-estimate victim sample picks survivors by frequency rank. Ghost tie-breaks and probation length both fail this way.
- Neither the admission contest nor the victim sample ever reaches stale-high residents. The contest only faces probation, and protected keys leave only through demotion. Jitter therefore just churns already-cold probation keys and swaps a higher-count key for a lower-count one.
- Recency evidence, whether ghost history or a longer trial, cannot act unless it moves the comparison by a margin or the counts themselves decay. This contradicts the cycle 0003–0004 verdict that aging is negative for workload-a. That verdict was reached on an earlier design (no SLRU, no dedup at first).
- v001's ghost ring adds metadata. Under hit-rate-only scoring that cost goes unpriced, and the ring is pure overhead if it never changes an outcome.

## Open questions

- Would a margin-based ghost admission (ghost hit plus a small bonus beating the victim) change outcomes, or does it just recreate the tie-admission churn that failed before?
- Is aging negative on the current dedup+SLRU design too, or only on the older ones? Note that cycle 0005's ≤ 1 fast-path shortcuts must be removed if aging returns.
- Can a recency signal be applied to protected-segment demotion instead, where stale-high keys actually live?

## Continuation

Drop exact-tie tweaks, random admission jitter and SLRU ratio sweeps. They are inert or negative. If hit-rate work continues, run at most one decisive test: either periodic halving on the current SLRU+dedup design, to re-check the old anti-aging verdict, or a ghost-hit margin in the contest. Otherwise, treat workload-a as at its ceiling for this family, and spend effort on getting per-op-time and metadata-size scored rather than on more hit-rate tweaks.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
