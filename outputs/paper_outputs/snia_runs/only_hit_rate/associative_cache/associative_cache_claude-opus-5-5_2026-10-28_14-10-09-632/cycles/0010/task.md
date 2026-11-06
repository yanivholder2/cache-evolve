# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants built on cycle 0006 v003 (GCLOCK main with ceiling 7 + floored probation FIFO). v001 added a recent-hit bit and halving decay to main. v002 capped the promotion sweep and rejected promotion when main was hot. v003 made the one allowed depth test, raising the small-bucket floor between 64 and 96. v001 is the Winner. It gained most on web_0 at the large size, which had been stuck, and only just cleared the bar on its target, hm_0 at the large size. It cost a little on hm_0 at the small size. v002 gave bit-identical results to the parent, and v003 regressed both small-bucket cells.

## Patterns

- Recency decay in main helps by clearing out stale blocks, not by keeping near-miss blocks. Halving gated on recency cleared once-hot web_0 blocks faster and never hurt long-lived ones. But it barely moved hm_0, so the cycle 0006 "missing recency signal" theory explains only a small part of hm_0's large-size gap. Small buckets gained nothing, because their short main sweep already decays stale counters quickly.
- Coarser ranking costs capacity in small buckets. Bit-length ranking merges counter tiers, which undoes part of what ceiling 7 gained in the small main region. This matches the cohort-wide finding that small buckets depend on main's capacity and how finely it ranks, and they lose whenever either one shrinks.
- With reset-to-0, main always has cold victims. Every promotion puts a counter-0 block into main, and most web_0 promotions are never hit again. So the "hot main under promotion pressure" case almost never happens, which refutes H02's premise. Admission gates keyed on main being full of hot blocks are a dead end. The cycle 0006 worry about worst-case sweep cost was also mostly theoretical.
- Probation depth is closed. v003 met both refutation conditions. With ceiling 7, smaller main did not become more space-efficient, so 64 is a local optimum. This makes three cycles of evidence that probation depth is used up.
- Main-side levers now look nearly used up too: depth, admission strength, counter ceiling, admission gating and recency weighting have each been tried. The remaining hm_0 large-size gap most likely sits outside main: which blocks get promoted, or the near-miss history that cycle 0005's ghost captured.

## Open questions

- Can a small-bucket fix bring back fine tiers under the recency rule, for example ranking by recent bit plus raw counter instead of bit length, without giving up the web_0 large-size gain?
- Among main's counter-0 blocks, would choosing the victim by age or by whether it was a fresh promotion beat choosing the first one from the hand, given that main's victims are almost always fresh promotions?
- Can near-miss history be used for something other than admission (cycle 0005 refuted admission because it re-admits web_0 single-return blocks), for example as a tie-break among probation evictions?
- Did v002's evaluation really run the overlay? The code analysis predicts identical results, but a build-reuse check is worth doing once.

## Continuation

Make cycle 0007 v001 (recency bit + halving decay) the parent, and keep reset-to-0 on promotion. Next, try to recover small-bucket tier resolution under the recency rule, and look at which counter-0 block main evicts or which probation hits get promoted. Do not test probation depth again, or admission gates conditioned on main being hot. Higher ceilings, ghost admission and non-zero promotion counters stay retired. Per-op-time is still unmeasured, though v001 reduces the worst-case sweep.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants built on cycle 0007 v001 (GCLOCK ceiling 7 + recent bit + halving decay, floored probation) and kept reset-to-0. v001 used linear decay in small main regions only. v002 added a one-shot "veteran" grace bit for blocks hit in main. v003 stopped counting probation hits that arrive in a correlated-reference window as promotion credit. v003 is the Winner: it gave the lineage's largest single-cell gain since probation was added (web_0 at the large size) and nudged hm_0 up at both sizes. Its only weak cell was web_0 at the small size, which was flat to slightly down. v001 and v002 both helped and hurt nothing.

## Patterns

- For the first time, a cohort had no real regressions. All three changes were safe, and each worked on a different path: v001 on small-main decay, v002 on the order of main's counter-0 victims, v003 on the probation promotion signal. They do not overlap, so they are candidates to combine, not alternatives.
- The levers depend on scale, and each acts at only one size. v001 changed only the 64-slot main (the large cells were bit-identical). v002 and v003 mattered mainly in large buckets, because the grace visit and the window are both too short to matter in small ones. Changes should be sized in absolute slots per region, not as one rule or ratio for every size.
- Tightening admission keeps winning. v003 is the fourth tightening of who reaches main to succeed (after probation, the depth floor and reset-to-0). It is also the first change to the promotion signal itself that worked. Burst re-references on web_0 were a meaningful share of the wasted counter-0 promotions that cycle 0007 identified.
- web_0 gains more from main-side history than expected. v002's veteran bit helped web_0 at the large size more than hm_0, which refutes the idea that web_0's decayed blocks are mostly stale.
- hm_0's large-size gap is still mostly unexplained. Recency (cycle 0007), victim ordering (v002) and promotion filtering (v003) each moved it only a little. Ranking inside main now looks nearly used up for hm_0.
- Small buckets reward finer ranking. v001's recent bit plus linear counter confirmed cycle 0007's tier-resolution diagnosis without hurting web_0. Note that v002's insight blames the small-bucket cost on "v001", but it means cycle 0007 v001, not this cycle's v001.

## Open questions

- Does widening v003's window in 128-slot buckets (for example an absolute floor of about 16 slots) bring the web_0 gain to the small size, and at what width does hm_0's real reuse start being filtered?
- Do v001, v002 and v003 stack? They act on different paths and sizes, but v002's grace bit and v003's filter both target counter-0 promotions in large main regions, so their gains may overlap.
- What drives hm_0's remaining large-size gap? It is likely long-distance reuse that probation never sees.

## Continuation

Make cycle 0008 v003 the parent. First try folding in v001's size-gated linear decay, which should be close to free because it touches only small buckets. Also widen the correlated-reference window in small buckets using an absolute floor. Test v002's veteran bit on top of v003 as a separate variant, because the two may overlap. Do not revisit probation depth, hot-main admission gates, higher ceilings, ghost admission or non-zero promotion counters. Per-op-time is still unmeasured.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants built on cycle 0008 v003 (GCLOCK ceiling 7 + recent bit + halving, floored probation, correlated-reference filter, reset-to-0) and tested the three combinations that cycle 0008 proposed. v001 folded in linear decay, gated to small main regions, and is the Winner. It lifted both 16384 cells, including web_0 at the small size, which had been v003's weak cell, and left the 131072 cells bit-identical. v003 (the veteran grace bit on top of the filter) helped a little everywhere and hurt nothing. v002 (a wider small-bucket window) traded web_0 for hm_0 at the small size and was not confirmed.

## Patterns

- The size-gated split now works as intended. Small buckets want fine linear tiers and large buckets want halving. Under the filter, linear decay gained more than it did on its own: once burst-only blocks are kept out of main, the finer tiers are spent ranking blocks that really are reused.
- The two changes aimed at counter-0 promotions overlap. The veteran grace still adds something on top of the filter, but only part of what it gained alone, because the filter has already removed many of the fresh promotions the grace used to outrank. It still costs nothing, though, so it is worth keeping.
- The correlated-reference window does not scale as an absolute width. Sixteen slots is a small fraction of the large probation ring but a quarter of the small one, and at that width it starts to filter hm_0's real short-gap reuse. The window is close to what it can do in small buckets.
- Small-bucket web_0 is no longer limited mainly by bursts. The lever that moved it this cycle was main-side ranking (linear tiers), not the promotion filter.
- Large buckets moved only through v003's grace. The large-size hm_0 gap is still largely unexplained.

## Open questions

- Does v003's veteran bit stack with v001's linear decay? In small buckets both act on main ranking, so the grace's small-size gain may vanish. Its large-size gain should survive, because v001 does not touch large buckets.
- Is there a small-bucket window strictly between the current narrow one and v002's quarter-ring width that filters web_0 bursts without catching hm_0 reuse, or is the window used up at that size?
- What still limits hm_0 at the large size? Main ranking, recency and promotion filtering have each moved it only a little.

## Continuation

Make cycle 0009 v001 the parent, and first add v003's veteran bit on top of it. Expect gains mainly at 131072, and check whether the small-size gain survives under linear tiers. Stop widening the small-bucket correlated window. At most, test one intermediate width, and do not use an absolute 16. Keep reset-to-0. Probation depth, hot-main admission gates, higher ceilings, ghost admission and non-zero promotion credit all stay retired. Look for a new lever for large-size hm_0 rather than tuning main ranking further. Per-op-time is still unmeasured, and linear decay brings back a longer worst-case sweep in small buckets.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
