# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Two variants built on cycle 0002 v002 (LRU + bimodal insertion). v001 drove the MRU-sample rate with a PSEL counter fed by per-item tags. v003 added graded two-step promotion. The third, v002, put an S3-FIFO-style probation ring in front of the cycle 0002 GCLOCK bucket. v002 is the Winner. It improved hm_0 at both sizes and gave web_0 at the large size its biggest gain, beating that hypothesis's refutation threshold. web_0 at the small size stayed flat. This is the first cycle since 0001 where the large-bucket cells clearly moved.

## Patterns

- Cold insertion carries over to CLOCK, and it works better there than in the LRU lineage. In v002, a new item only gets into the frequency-ranked main region if it is hit while in probation, and the main sweep only runs on promotions. That filtered scans on web_0 and also kept hm_0's reused blocks protected, so it is the first change to help both traces without a trade-off. The two lineages have now traded places: GCLOCK+probation leads, and the LRU/BIP line is stuck.
- Both LRU-lineage variants traded one trace against the other again, and each failure came down to mechanism detail, not the idea itself. v001's PSEL fixed cycle 0003's lost-signal problem: per-item tags last for the item's whole lifetime. But the counter's symmetric ±1 update, starting at mid-range, votes on whether *most* samples hit, not on whether sampling pays off. So it drifted toward LIP and gave back hm_0's gain. In v003, every lower-half hit landed in one shared slot, so the protected half was cut off from the reuse stream.
- Every variant still runs into the same limit: fixed-fraction probation is too short in 128-slot buckets for web_0's late second touches. That held for the LIP lineage in cycles 0001–0003, and it now holds for v002's `len/4` ring too.
- For the fourth cycle in a row, per-op-time went unmeasured, even though all three hypotheses asked for it.

## Open questions

- Would a probation ring that is deeper in small buckets, or that adapts its size (S3-FIFO-style ghost or set dueling on `small_len`), recover web_0 at the small size without weakening the filtering in large buckets?
- Should promoted items enter main with a non-zero counter, so they are not the next sweep victim?
- Can v001's tag-based PSEL be saved with an asymmetric update, a start in the higher-sampling band, and a reward test that also counts promotions, or should the LRU/BIP line be retired?

## Continuation

Make cycle 0004 v002 (GCLOCK + probation FIFO) the main lineage. Tune or adapt `small_len`, aiming at web_0 at the small size, and try giving promotions a non-zero starting counter. At most one LRU-lineage variant should be kept: the asymmetric-PSEL fix. Drop graded promotion through a single boundary slot. Per-op-time must be reported, or the variant is not accepted.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle 0004 v002 (GCLOCK main + probation FIFO), each testing one knob in isolation. v001 added a main-sized ghost ring for warm admission. v002 had promoted items carry a non-zero counter into main. v003 added an absolute probation-depth floor. v003 is the Winner. It was the only variant with no regressions, and it improved both traces at the small size, while the large-bucket cells stayed exactly the same as the parent.

## Patterns

- Depth helped, and both admission-strength changes hurt. Whenever the bar for entering main got lower or main got more counter mass (v001 by ghost match, v002 by carried counter), main churned more and established blocks decayed faster, mostly on web_0. Only a hit during probation is a reliable promotion signal. Only adding probation depth helped, because it gives more blocks the chance to earn that hit.
- In this lineage, the probation/protected trade-off is not zero-sum, which is the opposite of the LRU line. Main in small buckets shrank to half, and hm_0 still gained. A frequency-ranked main whose counters are decremented only on promotion needs fewer slots than an LRU protected segment. This removes the cycle 0002–0003 rule that "hm_0 wants protected and web_0 wants probation" for this design.
- Ghost admission has now failed in two lineages and at two sizings (cycle 0003 undersized, cycle 0005 main-sized). With enough coverage it brings back hm_0's near-miss reuse but re-admits web_0's single-return blocks. This is a trace split, not a sizing problem.
- The promoted block's position behind the hand already gives it a full rotation of grace. Adding a counter on top of that is redundant and harmful. This settles cycle 0004's open question with a no. v003's own takeaway still lists a non-zero promotion counter as a candidate, but v002 refuted that idea.
- The web_0 gain at the small size is real but modest, so depth explains only part of the gap. Nothing in this cohort moved the large-bucket cells upward.
- Per-op-time went unmeasured for the fifth cycle in a row. The insights say it is absent from the fitness vector itself, so this is a harness gap, not a variant omission.

## Open questions

- Does pushing the floor further (beyond half the bucket in small buckets) keep helping both traces, and where does hm_0 start to lose main capacity?
- Does a deeper probation share at the large size (such as a fraction above a quarter in 1024-slot buckets) help web_0 there, or does it weaken the filtering that cycle 0004 gained?
- What explains the rest of web_0's small-size gap, if not probation depth or admission strength?

## Continuation

Make cycle 0005 v003 (GCLOCK + probation with a depth floor) the parent. Sweep probation depth as the main lever: a higher floor in small buckets and a larger fraction in large buckets, possibly adapted with set dueling on `small_len`. Retire ghost/warm admission and non-zero promotion counters, and keep the reset-to-0 on promotion. Stop requiring per-op-time from variants until the harness adds it to the fitness vector, and flag that gap to whoever owns the harness.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
