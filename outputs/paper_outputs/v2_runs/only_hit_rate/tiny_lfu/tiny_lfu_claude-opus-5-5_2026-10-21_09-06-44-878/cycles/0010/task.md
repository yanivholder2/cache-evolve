# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0010

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants tried frequency-only levers that had not been tested before, with no aging. v001 capped the sketch counters at 31. v002 shrank the victim sample from 8 to 3. v003 let a probation hit enter protected only if its estimate beat the demotion candidate. All three left workload-a hit-rate flat within noise. v003 is the Winner only as a tie. This is the fifth cycle in a row with no real gain.

## Patterns

- Leaving count decay aside, every remaining lever is now inert. The cap, the sampling depth and the promotion gate all change comparisons that the min-estimate sample and probation-MRU placement already settle the same way. This confirms the earlier view that the design is over-defended.
- Sampling depth works against itself. A shallower scan spares young keys, but it raises the victim estimate, so fewer newcomers get admitted. The two effects cancel, so depths between 3 and 8 look equivalent.
- SLRU segment movement is now fully covered: demotion choice, split ratio, idle demotion and now promotion gating. None of them moved hit-rate. Which keys sit in protected is not the binding constraint on this trace.
- The cap result is the cohort's one usable output. Counts above 31 carry no decision-relevant ordering, so 5-bit counters should be free on hit-rate. That only pays off if metadata-size is scored, which was still unconfirmed as of cycle 0008.
- v002's takeaway recommends aging or halving the counters next. Cycle 0008 tested these directly on the current design and they lost. Treat that suggestion as already refuted.

## Open questions

- Is metadata-size (or per-op-time) actually on the leaderboard now? If so, are 5-bit counters (cap 31) neutral as predicted? A 4-bit cap (15) is untested and might change comparisons.
- Is workload-a's plateau simply the trace's ceiling for TinyLFU+SLRU, given that the admission contest, the victims, the segments and aging have all been tested?

## Continuation

Stop proposing workload-a hit-rate tweaks to the sketch range, victim sampling depth, SLRU promotion/demotion or aging. All of these are now tested and inert or negative. The only open lead is memory: if metadata-size is scored, pack counters to 5 bits (cap 31) and expect a tie on hit-rate. Otherwise, spend effort on getting cost dimensions scored or on other workloads.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
