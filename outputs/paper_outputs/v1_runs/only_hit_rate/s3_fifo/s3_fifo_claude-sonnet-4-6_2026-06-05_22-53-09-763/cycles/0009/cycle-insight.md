# Cycle 0009 Insight

## Cohort outcome

All three variants continued exploiting the small-queue sizing and main-queue protection axes identified in cycle 0008: further shrinking the small queue, stacking MTF on main hits onto the prior base, and extending the eviction recycle window. Only the further queue reduction landed as a winner; MTF on the older base was a narrower positive that didn't surpass it, and the recycle extension was a small loss.

## Patterns

- **The small-queue sizing benefit continues below ~7% without degrading the cold filter.** Reducing to ~5% still admitted enough cold traffic rejection that main gained higher-quality residents — the monotone trend from cycle 0008 holds at least one step further, and the floor remains unfound.
- **Queue sizing and MTF address orthogonal failure modes.** Cold-admission pressure (mitigated by a smaller small queue) is the larger drag on this workload; positional drift in main (corrected by MTF) is a secondary inefficiency. The two mechanisms combine additively rather than substitutively, but queue sizing dominates.
- **The freq==1 recycle extension is ruled out under eager promotion.** Items that reach the main-queue tail at freq==1 after eager promotion are genuinely cold on Zipfian access; granting them an extra pass delays fresher admissions without improving their own hit probability. The existing freq>1 threshold is already well-matched to the post-promotion frequency distribution.

## Open questions

- Is there a true floor to the small-queue sizing benefit, and does it manifest as a sudden quality cliff when the filter becomes too coarse, or a gradual diminishing return?
- Would combining the ~5% queue size with MTF on main hits yield an additive gain exceeding either mechanism alone, given that they target orthogonal failure modes?

## Continuation

The highest-priority probe for the next cycle is pushing the small queue below ~5% (e.g., capacity/28 or capacity/40) to locate the floor empirically. Simultaneously, combining the winner's queue size with MTF is worth one explicit test — both mechanisms are confirmed positives addressing different failure modes and their interaction hasn't been measured at the ~5% base. The recycle-window axis (freq threshold on evict_main) is closed; do not revisit it.
