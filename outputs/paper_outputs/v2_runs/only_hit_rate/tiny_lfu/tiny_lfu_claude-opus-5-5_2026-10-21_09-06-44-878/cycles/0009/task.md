# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0009

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle ran the decisive recency tests that cycle 0007 called for, each on a different parent. v001 added global periodic halving to SLRU+dedup. v002 let a ghost-hit newcomer beat the victim by a margin. v003 discounted idle residents and demoted them out of protected. All three lowered hit-rate on workload-a. v002 is the Winner only because its loss was the smallest, close to noise. This is the fourth cycle in a row with no real gain.

## Patterns

- All three ways of adding recency failed: decaying counts (v001), recency evidence in the contest (v002) and per-entry idle decay (v003). On workload-a, lifetime frequency is the asset. The keys it protects tend to have long reuse distances, and they are worth more than the stale-high keys that recency would remove.
- The sizes of the losses match how widely each change acts. Halving every count, or discounting every sampled victim, touches every eviction. v003's one-sided discount (victims discounted, newcomer not) pushed admission toward LRU and lost TinyLFU's scan resistance. v002's margin applies only to ghost keys seen at least 3 times, so its loss was tiny.
- Cycle 0007 asked whether old results against aging held only for older designs. They do not. v001 was a clean test on the current design and still lost, so it rules out the earlier confounders.
- Cycle 0007 also asked whether recency could act through protected-segment demotion. The answer is no: the protected segment already keeps keys that return just past the idle thresholds tried here, and demoting them on idle time evicts them before their next reuse.
- Two variants carry caveats. v003 bundled two mechanisms, so the cost of each is not separated. v002's author could not inspect the source, so it is unconfirmed that the change was implemented as specified.

## Open questions

- Would protected-head demotion alone (raw estimates, no discount) be neutral or still negative? This is the only part not tested on its own.
- Would a much longer halving period (well beyond 10×capacity) avoid the damage? This would be a last check, and probably converges to the unaged parent.

## Continuation

Treat workload-a's hit-rate as at its ceiling for the TinyLFU+SLRU family. Drop aging, ghost margins, idle discounts and idle-based demotion. At most, spend one variant on very long-period halving or raw-estimate demotion alone, and only to close the question. Spend remaining effort on getting per-op-time and metadata-size scored, or on other workloads, rather than on more workload-a hit-rate tweaks.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
