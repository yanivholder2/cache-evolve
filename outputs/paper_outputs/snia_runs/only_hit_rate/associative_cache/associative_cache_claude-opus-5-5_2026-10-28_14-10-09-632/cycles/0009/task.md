# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0009

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants built on cycle 0005 v003 (GCLOCK main + probation FIFO with a depth floor). Two of them pushed probation depth further: v001 raised the fraction in large buckets and v002 raised the floor in small buckets. Both lost hit-rate. The third, v003, raised the main counter ceiling from 3 to 7 and is the Winner. It improved hm_0 at both sizes, most of all at the large size, which had been stuck since cycle 0004. web_0 edged up at both sizes, so nothing regressed.

## Patterns

- Probation depth is now bracketed and used up as a lever. Both depth variants failed on the trace they targeted, and the cost fell on main capacity in both cases. The "not zero-sum" result from cycle 0005 holds only up to about half the bucket in small buckets and at the existing quarter fraction in large buckets. Past those points, adding depth takes slots away from a main region that both traces need.
- The only gain came from the main side: how well main ranks its residents, not how big it is or who gets in. This is the first change since cycle 0004 to move a large-bucket cell, and it moved hm_0 without a web_0 trade-off.
- The reset-to-0 on promotion is now the key safety property of this lineage. It stopped ghost and carried-counter admission from hurting in cycle 0005, and here it stops a higher ceiling from trapping stale web_0 blocks. Any new main-side state should keep it.
- v003 reports that the gain is small and that frequency resolution is close to used up. The remaining hm_0 large-size gap (compared with cycle 0005 v001's near-miss retention) looks like a recency signal, which a counter cannot capture.
- v002's takeaway suggests a non-zero promotion counter and ghost sizing as next levers. Cycle 0005 refuted both, so those suggestions should not be followed.
- Per-op-time is still unmeasured. v003 raises the worst-case promotion sweep, so this cost is now growing without being seen.

## Open questions

- Can main add a recency signal (for example a recent-hit bit that sweeps clear before they decrement frequency) to recover hm_0's near-miss reuse without admitting web_0's single-return blocks?
- Is there a better small-bucket split strictly between 64 and 96 probation slots, or is the gain there too small to be worth pursuing?
- What still limits web_0 at the large size, now that depth, admission strength and counter resolution have all been ruled out?

## Continuation

Make cycle 0006 v003 (GCLOCK + floored probation, ceiling 7) the parent. Stop sweeping probation depth: keep the floor at 64 and the large-bucket fraction at a quarter, and allow at most one fine-grained test between 64 and 96. Focus on main-side victim selection, in particular adding recency to the frequency counter, while keeping reset-to-0 on promotion. Do not try higher ceilings, ghost admission or non-zero promotion counters again. Keep flagging the missing per-op-time measurement, now that sweep cost has grown.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
