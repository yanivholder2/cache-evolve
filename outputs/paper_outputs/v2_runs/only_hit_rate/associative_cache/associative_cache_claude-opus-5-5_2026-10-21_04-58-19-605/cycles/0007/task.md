# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants were narrow follow-ups to Cycle 0005's continuation. v001 shrank the GCLOCK ghost ring from 4 to 2. v002 retried the leniency duel with DIP-style fixed leader buckets and a global PSEL. v003 limited ghost recording on the 1/8 LRU line to keys that were never hit. All three hypotheses were refuted and no variant beat its parent. v002 is the listed winner only because it lost the least. It adds no new mechanism worth keeping, and its own insight argues for dropping the duel.

## Patterns

- Every tweak to the ghost signal lost a little, so the current settings are already locally optimal. On ring size, 2 lost slightly and 16 (Cycle 0005) lost more, so the peak is at or near 4. On ring admission, filtering out previously-hit keys removed useful ghost hits on the LRU line. Ring length, ring admission and newcomer grace have now each been probed from both sides on workload-a.
- Newcomer leniency is closed on workload-a. Even with fixed leaders that removed the self-referential bias from Cycle 0005, the lenient one-sweep grace did worse than strict hand-on-newcomer. The duel also costs hits just by keeping lenient leaders running. Capped ghost admission already gives returning keys all the protection they need.
- Lessons about ghosts do not carry over cleanly between lines. Cycle 0005's "narrow the ring" result came from CLOCK, where long-reuse keys diluted the ring. On LRU, a hit key that has aged to index 0 has a reuse distance just past the bucket, which is exactly the population the ghost reward is meant to rescue. The two lines' ghost rings select different populations, so the same filtering rule behaves differently on each.
- All losses this cycle were small, and some may be within noise on one trace at one size. Workload-a hit rate looks saturated for the current design family.
- After six cycles, per-op-time and metadata-size are still not recorded. That is why v001's real saving (half the ghost memory, half the miss-path scan) counted for nothing. With hit rate saturated, these unmeasured dimensions are now what decides between candidates.

## Open questions

- Does a 3-entry ring match 4? This only matters once metadata-size or miss-path cost is measured.
- Does strict filtering plus a capped ghost counter hold up on a second trace or cache size, especially one with longer reuse distances?
- On the LRU line, would changing ghost-hit insert depth or ring length help where changing ring admission did not?

## Continuation

Treat strict hand-on-newcomer plus a 4-entry capped ghost ring (cycle/0004/v003) as the settled CLOCK-line shape. Drop newcomer-leniency duels, ghost-ring admission filters, and rings larger than 4. Do not spend another cycle on single-knob hit-rate tweaks on workload-a. The next cycle should get per-op-time and metadata-size recorded, or a second trace or cache size into the evaluation, and then compare the CLOCK+ghost and 1/8 LRU+ghost lines (and a 2- or 3-entry ring) on those axes. If the hit-rate search continues, the only lever left untested on the LRU line is ghost-hit insert depth.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
