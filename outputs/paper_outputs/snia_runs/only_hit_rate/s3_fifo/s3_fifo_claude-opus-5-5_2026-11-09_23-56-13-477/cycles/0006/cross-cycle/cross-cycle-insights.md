# Cross-Cycle Insights

## Recurring wins

- Per-key evidence beats global signals. Gating on what each key carries with it (its small-queue freq when it was ghosted, then its ghost age) gave the only clean policy gains in the lineage (cycles 0004–0005). Every global proxy failed: main occupancy, ghost-return rate, and the unweighted ARC split.
- The strict two-reuse admission into main, plus the freq reset, has held up as the base in every cycle. Loosening it has never survived steady-state pressure.
- Ghost age works as a floor for freq-0 returnees, and about one small-queue turnover is the right boundary. A quick return marks hm_0's real reuse; a late return marks web_0's one-off rereferences.
- Mechanisms whose windows scale with `small_size` can move cells at the large cache size. Fixed or global knobs only ever moved the small size.
- Cost-only refactors (the O(1) index/slab, fast hasher, compact `Loc`) have stayed bit-identical in hit-rate every time. They are safe to port onto any policy parent.

## Recurring failures

- Single-constant tweaks to the threshold or to the static small/main split. hm_0 and web_0 split in opposite directions, and the effect flips sign across cache sizes (cycles 0001–0002).
- Rewarding ghost returnees through global signals or flat credit, such as rate-switched thresholds or second-chance credit. The ghost mostly holds weak keys, and main is always full, so any extra protection evicts entries that would have been hit (cycles 0003, 0006).
- Relap stacking. Resetting freq erased the ghost signal (cycle 0005). Keeping freq 1 made the relap redundant with the ghost path and cost small-queue capacity (cycle 0006).
- Re-admitting keys to small at the small cache size. Displacing residents keeps costing more than the main pollution it avoids.
- Loosening the age window beyond one turnover, or adding age ceilings on freq ≥ 1 returnees. Both give back hit-rate on web_0.
- Measurement gap: per-op-time and metadata-size have been missing from the fitness vector for six cycles in a row. Hypotheses that assumed these were reported were evaluated on hit-rate alone.

## Open questions

- What limits web_0 at the large size? Every ghost and small routing change has left it flat, whether it tightened or loosened admission. Candidates are main's eviction order, the split at that size, and first-admission filtering of never-seen keys.
- Would a window narrower than one turnover sharpen the hm_0 / web_0 split?
- How much do the pending cost-only variants actually save in per-op-time and metadata-size, once those are measured? Alignment may cancel the layout savings on 8-byte keys.
- Is a direct comparison of what promoting a key gains against what displacing a main entry costs workable as a gate?

## Per-policy notes

- S3-FIFO (the only policy in this lineage): the current best is cycle 0005's v002, the age-gated ghost path (freq-gated, age window at 1×) on the O(1) index. Ghost and small routing looks locally optimal and close to exhausted after cycle 0006. Gains have been small and mostly at the small cache size.
- hm_0 is lenient-friendly at the small size only; its earlier preference for a looser threshold was largely a cold-start effect. web_0 punishes any loosening of admission, and at the large size it has not moved under any change so far.
- Next directions: target main's eviction and first-admission handling, and port the cost changes onto the policy parent. Push for the cost dimensions to be measured before spending more hypotheses on cost.
