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
