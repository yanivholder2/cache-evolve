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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
