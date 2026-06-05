# Variant v002 Insight

## Helped

- (no measured dimension improved; capping `p` at `p_cap = capacity / 20` in the B1-ghost branch left hit-rate exactly where the uncapped parent sat.)

## Hurt

- (no measured dimension regressed; the cap is a ceiling on an upward adjustment, so it can only ever reduce `p`, never destabilize the existing T1/T2 bookkeeping.)

## Takeaway

Clamping ARC's adaptation target `p` to ~5% of capacity did not reproduce the 95/5 SLRU incumbent's lift — hit-rate stayed pinned at the uncapped value. On this reuse-heavy trace, `p` evidently never climbs far enough above the 5% ceiling for the clamp to bite, so ARC's gap to the SLRU split is not caused by an over-large recency tier. The live thread the cross-cycle insight named is now ruled out: ARC's ghost-driven `p` is not the lever; its underperformance must lie elsewhere (e.g. T2's internal ordering or admission, not its size).
