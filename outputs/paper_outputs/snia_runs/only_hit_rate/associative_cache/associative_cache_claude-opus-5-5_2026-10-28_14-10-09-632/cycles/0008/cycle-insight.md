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
