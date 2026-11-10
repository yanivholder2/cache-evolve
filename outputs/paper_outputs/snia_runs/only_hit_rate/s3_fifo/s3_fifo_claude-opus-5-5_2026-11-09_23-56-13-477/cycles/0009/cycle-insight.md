# Cycle 0009 Insight

## Cohort outcome

Two variants ran cycle 0008's suggested follow-ups. v001 tiered 0008-v003's doubled ghost. v002 scaled 0008-v001's `main_ghost` budget up to `main_size`. v003 took the first step off the ghost path: it applied a correlated-reference cap on small-queue hits that arrive soon after insertion, built on 0005-v002. v003 won on hm_0, gaining at both sizes. It lost at the small size on web_0 and was flat at the large size there.

## Patterns

- The hm_0 / web_0 trade-off held again, now for a fourth cycle in a row. Every lever helped one trace at the other's expense, and each variant's gain showed up only at the small size, or mostly there.
- Both of 0008's follow-ups failed. Tiering removed direct-to-main entry for old freq ≥ 1 ghosts, and it turned out that privilege was part of hm_0's small-size gain. Ghost age alone does not separate useful late returnees from noisy ones. A three-times-larger `main_ghost` left both large-size cells unchanged. That refutes 0008's claim that a horizon of about `main_size` for hit main victims explains web_0's large-size plateau.
- v003's gain comes from deferring main admission, not rejecting it. A key whose small-queue hits all fall in its insertion burst keeps freq 1 into the ghost. The freq ≥ 1 ghost→main path then admits it on a later, spaced return. hm_0 has burst-only keys that pollute a full main. On web_0, a quick reuse after insertion is real evidence.
- web_0 at the large size has now resisted ghost routing, ghost age, ghost budgets, the split, main-victim memory and burst filtering. Cycle 0001's web_0 pollution does not come from burst-only keys.
- Per-op-time and metadata-size are still not measured, for the ninth cycle running.

## Open questions

- Would a narrower correlated-reference window (a larger `CRP_DIVISOR`) keep hm_0's gain and spare web_0's quick reuses?
- For web_0 at the large size, two candidates are left. Which main victims get ghosted at all (the `main_hit` gate)? How do main-ghost returnees get admitted (currently freq 0, straight into main)? Or is that cell simply at its ceiling for this policy family?

## Continuation

Use 0009-v003 (the correlated-reference cap on 0005-v002) as the policy base. Spend one hypothesis on narrowing its window. Drop ghost tiering by age, `main_ghost` budget scaling and uniform ghost-budget scaling, because those axes are exhausted. Spend at most one probe on web_0 at the large size, through the `main_hit` gate or the freq given to main-ghost returnees. If that probe also fails, treat the cell as a plateau and stop targeting it.
