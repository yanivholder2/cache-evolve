# Cycle 0010 Insight

## Cohort outcome

This cycle tested three focused hypotheses: stacking MTF on main-queue hits onto the ~5% small-queue base, shrinking the small queue one step further to ~3.6%, and redirecting ghost population from cold evictions to main-queue evictions. The winner combined the ~5% queue with MTF — confirming the additive interaction that cycle 0009 left unmeasured.

## Patterns

- **MTF and queue sizing compound additively, and their combination beats further queue shrinking alone.** Both mechanisms target distinct failure modes on this workload, and stacking them at the ~5% base outperformed the next queue-size reduction step. The two levers have now been jointly confirmed as the new best baseline.
- **The small-queue benefit curve is flattening but has not cliffed.** The ~3.6% step still improved hit rate, but the incremental gain was visibly smaller than the ~5% step, and it was not strong enough to win over the MTF combination. The true floor is close but not yet breached.
- **Ghost signal value is driven by volume, not semantic quality.** Redirecting ghosts to track main-queue evictions instead of cold small-queue rejections collapsed coverage: main evictions are sparse and describe items whose working-set tenure is already exhausted, so almost none of the re-accessing medium-popularity keys got fast readmission. This axis is definitively closed.

## Open questions

- Is the small-queue floor a gradual diminishing return or a sudden quality cliff, and does it fall between ~3.6% and some smaller fraction?
- Would combining MTF with the ~3.6% queue (rather than ~5%) yield a further additive step, or has the positional-drift failure mode already been largely eliminated by the time the smaller queue promotes items?

## Continuation

The new baseline is ~5% small queue plus unconditional MTF on main hits; the next cycle's hypotheses-writer should treat this as the starting point. The most informative remaining probe is one more queue-size reduction step (e.g., capacity/40) combined with MTF, to test whether the additive property holds at a tighter admission filter and to locate the floor empirically. The ghost-source axis and the recycle-window axis are both closed; do not revisit them.
