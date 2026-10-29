# Cycle 0008 Insight

## Cohort outcome

Two variants split cycle 0007 v003's miss-dedup into its two coupled effects: v001 restored the aging clock on the incumbent line, and v002 added a guarded admission tie-break on top of v003. v003 tried a CLOCK reference bit for protected hits to cut tail latency. All three are named Winners, but only v002 is a real one. It raised hit-rate in all four cells and has no hurt entries. v001 came in slightly below its parent on aggregate hit-rate, and v003 regressed both latency and hit-rate.

## Patterns

- The two-variable split worked, and it changes the picture of the dedup. Restoring the aging clock (v001) recovered most of the large-size web_0 loss but turned the small-size hm_0 gain into a loss. Restoring the tie edge (v002) recovered large-size web_0 while keeping and slightly extending the small-size gains. So the strict `>` tie-break, not only the slower clock, caused much of v003's large-size regression. And the dedup alone is not a win: the parent's miss→put double count was an implicit "seen again" bonus.
- A bonus for re-seen keys works best when it is explicit and bounded. v002's `>= 2` guard gives ties to re-referenced candidates and still keeps one-hit wonders out. That serves small and large sizes at once, which resolves for now the small-vs-large split seen since cycle 0004.
- v002's `get`/`put` are byte-identical to its parent, yet every latency percentile improved sharply. This mirrors v003's unexplained slowdown in cycle 0007 in reverse. Per-op-time noise between runs is still large enough to swamp real effects. Cycle 0007 v003's latency regression should now be treated as noise, not as a cost of dedup.
- Bits packed into link words hurt on this layout. The parent's protected-hit relinks were blind stores that the store buffer absorbed. Keeping a flag inside `next` turned every relink into a load-then-store, and the bounded second-chance loop multiplied relinks on promotion. The pointer-chase theory for get p99 has now failed three times (sketch prefetch, neighbour prefetch, and in-link CLOCK).

## Open questions

- Does v002's tie-break on top of the restored aging clock (v001's `tick()`) gain more than either one alone, or does the clock change undo the small-size gain?
- Is the `>= 2` threshold optimal, and would it hold at other cache sizes?
- Where does get p99 actually come from, if not from neighbour-link stalls? A reference bit in a separate slot-indexed bitset would show whether the problem is CLOCK itself or carrying the bit in `next`.

## Continuation

Adopt cycle 0008 v002 as the new base for hit-rate work. Next, test it combined with v001's clock-preserving `tick()`, and sweep the tie-win threshold. Drop dedup without a compensating bonus, and drop any flag packed into link words. If tail latency is pursued again, keep the reference bit outside `Node` and confirm per-op-time only with interleaved runs, since single-run latency verdicts went unreliable for the second cycle in a row.
