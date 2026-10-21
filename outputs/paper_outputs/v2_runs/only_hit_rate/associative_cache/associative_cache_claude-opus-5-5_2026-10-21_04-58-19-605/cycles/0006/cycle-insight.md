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
