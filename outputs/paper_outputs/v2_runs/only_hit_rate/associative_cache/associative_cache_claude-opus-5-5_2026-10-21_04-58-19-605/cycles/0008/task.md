# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0008

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Each variant took a different thread from Cycle 0006's continuation. v001 swapped the CLOCK-line ghost ring's stored keys for 16-bit fingerprints, a cost-only change. v002 inserted ghost hits on the LRU line at 3/8 depth instead of 1/4. v003 added a per-bucket miss-frequency sketch as a second admission signal on CLOCK. v001 won by matching its parent's hit rate exactly. H01 held on hit rate, but its real gain (about one-eighth of the ghost memory and a cheaper miss-path compare) still can't be seen, because the evaluation doesn't measure metadata-size or per-op-time. v002 slipped very slightly and v003 regressed clearly, so H02 and H03 were both refuted.

## Patterns

- The ghost ring only needs to answer "was this key evicted recently?" A membership test is enough, so a compact fingerprint ring loses nothing. This is the first change since Cycle 0004 that lowers cost on the settled CLOCK shape without moving hit rate.
- On LRU, ghost-hit insert depth peaks at 1/4. Going deeper gave nothing back, and the curve looks flat there. That was the last single-knob lever left on the LRU line. Depth, ring size, ring admission and insert depth have now all peaked at or before their current settings.
- The admission reward must stay rare. v003 kept the reward capped but widened the signal that grants it. Its small, shared, slow-ageing sketch saturated from hash collisions and ended up granting counter 1 to most newcomers, which in effect turned off the strict one-miss filter. Cycle 0004 found that a capped reward is necessary. This cycle shows it isn't sufficient: the trigger also has to stay selective.
- This is the second cycle running with no hit-rate gain on workload-a. The design family looks saturated on that trace. Seven cycles in, metadata-size and per-op-time are still unrecorded, and v001's win depends entirely on them.

## Open questions

- Would a miss-frequency signal with controlled error help? That would mean more counters, a short window, or a threshold that scales with the window. Or is the 4-entry exact ring already the complete admission signal for workload-a?
- Does fingerprinting the ghost ring work the same way on the LRU line?
- Once costs are measured, does the fingerprint ring (perhaps shrunk to 3 entries) make CLOCK+ghost the clear main line over LRU+ghost?

## Continuation

Adopt cycle/0007/v001 (CLOCK, strict newcomer filter, 4-entry fingerprint ghost ring) as the base going forward. Retire LRU-line insert-depth tuning and treat LRU+ghost as a reference line only. Drop broad frequency-based admission signals. Retry one only if its false-positive rate is bounded by design. Stop running hit-rate-only cycles on workload-a: getting metadata-size and per-op-time recorded, or a second trace or cache size added, is now required before any candidate can be told apart.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
