# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0006

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Two variants made the newcomer filter more aggressive: v001 moved insertion depth from 1/4 to 1/8, and v003 left the GCLOCK hand resting on the newcomer it just inserted, which gives a grace window of about one miss. v002 instead added frequency-aware victim choice within the bottom 4 slots of the 1/4-depth line. v001 won. v003 passed the previous cycle's depth winner and got close to v001, while v002's gain was barely above noise.

## Patterns

- For the third cycle running, how fast a newcomer must prove reuse is the main lever on workload-a. Both variants that strengthened the newcomer filter improved, from two different designs: shift-based LRU and in-place CLOCK. A filter as short as about 1–2 misses still did not hurt hit rate on this trace.
- Returns are shrinking along the depth axis: the step from 1/4 to 1/8 gained less than the step before it. Together with v003's near-LIP result, this suggests hit rate on workload-a is flattening near the eviction end.
- The CLOCK line's remaining gap was mostly about where the hand rests, not about frequency. Once the newcomer filter matched, in-place GCLOCK came close to deep-insertion LRU. Its no-shift design may make it the better choice once latency is counted.
- Frequency signal at the eviction end adds almost nothing when hits promote to MRU. This is the second cycle in which frequency-based or extra protection beyond the newcomer filter brought little or no hit-rate gain.
- Per-op-time and metadata-size are still not recorded, three cycles in. Every latency claim is still untested: v001's larger shifts, v002's window scan, and v003's expected O(1) miss path.

## Open questions

- Does pure LIP (index 0) help, or has v001's curve already reached its peak?
- Is the one-miss grace window over-fitted to workload-a? Traces with longer reuse distances or other cache sizes could penalise both v001 and v003.
- Once latency is measured, does v003's in-place design beat v001 overall despite a slightly lower hit rate?

## Continuation

Finish the depth bracket with pure LIP on v001, and test both v001 and v003 on at least one other trace or size to check whether the aggressive filter generalises. Make per-op-time recording a precondition. Without it, the cohort cannot be ranked between v001 and v003. Drop victim selection by frequency at the eviction end. If frequency is revisited, apply it on the promotion or insertion side, for example by setting insert depth from a key's count.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Each variant changed the newcomer filter in a different way. v001 made it stricter by moving the 1/8-depth line to pure LIP (index 0). v002 added a two-step, SLRU-style promotion on the hit side of the 1/8 line. v003 relaxed the GCLOCK line's one-miss filter for keys that recently returned, using a 4-entry per-bucket ghost ring that admits those keys with counter 1. v003 won: H03 held against its parent. v002 was flat, at noise level, and v001 regressed.

## Patterns

- The depth axis has peaked. Pure LIP lost hit rate, so the best tested depth is 1/8, which leaves about one slot of buffer behind the newcomer. On workload-a, a newcomer needs slightly more than one miss of grace. The aggressive-filter trend from cycles 0001–0003 stops here.
- The useful lever is now selective grace on top of a strict filter. Making the filter stricter for every key hurt (v001). Giving extra grace only to keys with evidence of a short reuse distance helped (v003).
- Ghost admission works only when the protection is capped. Cycle 0002 gave ghost hits a full MRU promotion and lost hit rate. v003 gives a single extra sweep pass and gains. The size of the reward matters as much as the signal that triggers it.
- Frequency on the promotion side added almost nothing (v002), because the 1/8 filter already removes most keys that are hit only once. This is the third cycle in which frequency-based protection beyond the newcomer filter did not pay off. v002's flag was also never reset after demotion, so the test was weak.
- Per-op-time and metadata-size are still not recorded after four cycles. v003 adds real cost to both: a ghost scan on every miss, and four keys per bucket. Its win is measured on hit rate alone, on one trace at one size.

## Open questions

- What ghost ring size is best: smaller, larger, or scaled to the bucket? Does the gain hold on traces with longer reuse distances?
- Does capped ghost admission carry over to the 1/8-depth LRU line? For example, a ghost hit could insert the key one step deeper than a normal newcomer. Does it help there as much as on CLOCK?
- With ghosts in place, the CLOCK line may now match or beat the 1/8 LRU line on hit rate. Once latency is measured, which line should be the main one?

## Continuation

Retire depth tuning: 1/8 is the optimum on the LRU line. Drop promotion by frequency, unless one cheap retry with the demotion flag reset properly is worth it. Build on v003: sweep the ghost ring size, try the same capped admission on the 1/8 LRU line, and validate on a second trace or cache size. Make per-op-time and metadata-size recording a hard precondition, because the CLOCK-plus-ghost lead now depends on miss-path and memory costs that have never been measured.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants extended Cycle 0004's capped ghost admission. v001 grew the GCLOCK ghost ring from 4 to 16 entries. v002 moved capped ghost admission onto the 1/8 LRU line, where ghost hits enter at index 3 instead of index 1. v003 added a per-bucket adaptive duel between strict and lenient newcomer grace on GCLOCK. v003 is the listed winner, but only because it tied its parent. It stays on the strongest line in absolute terms, yet H03 was refuted and nothing new was learned about leniency. v002 was the only variant to beat its parent, and only by a small margin. v001 regressed.

## Patterns

- The ghost signal works best when it is narrow. A small, recent ring is selective. Stretching it to cover keys with longer reuse distances diluted the counter-1 bonus, and those keys pushed out more valuable residents. Ring size behaves like the depth axis did in Cycle 0004: the peak is at or below the current setting.
- Capped ghost admission carries over between policy lines. It helped on both CLOCK (Cycle 0004) and shift-based LRU (v002). That supports the rule from earlier cycles that the size of the reward matters more than the signal that triggers it. The LRU gain is smaller, because the slot behind a 1/8 newcomer already catches most keys with short reuse distances.
- Adaptive controllers fail when they learn from their own outcome. In v003's duel, strict mode produced the negative evidence that kept the bucket in strict mode. The lenient branch almost never fired, so H03 was never really tested.
- This cycle produced no hit-rate gain on the CLOCK-plus-ghost line. Its lead now rests on Cycle 0004's result alone. Per-op-time and metadata-size are still not recorded after five cycles, and all three variants added miss-path scans or per-bucket state.

## Open questions

- Is a ring of 2–3 entries as good as 4, or better? This would also lower miss-path and memory cost.
- Would a properly built duel ever choose leniency? That needs set-dueling leader buckets, negatives counted only for newcomers admitted at counter 0, and a seed near the threshold.
- Is v002's small margin above run-to-run noise? Does it, or any CLOCK-line gain, hold on a second trace or cache size?

## Continuation

Shrink the GCLOCK ghost ring (try 2 and 3). Drop larger rings. Retry the adaptive grace duel at most once, with DIP-style leader buckets and an unbiased negative signal. Otherwise conclude that strict filtering plus capped ghost admission is the right shape. Keep ghost-on-LRU (v002) as a secondary line only. The hit-rate axis on workload-a looks close to saturated, so the next cycle should give priority to recording per-op-time and metadata-size and to validating on a second trace or size, ahead of further hit-rate tweaks.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
