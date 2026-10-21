# Cycle 0007 Insight

## Cohort outcome

All three variants were built on cycle-0006 v003 (rarer BIP). v001 removed BIP so that the ghost is the only blind-free route into protected space. v002 added a recency gate to ghost hits, honouring only those from about half the ghost window back. v003 reapplied cascade-free routing of displaced foreign residents to aim at per-op-time. v002 won. v001 also gained a little, but not enough to win. v003 matched its parent's hit-rate exactly, bit for bit.

## Patterns

- Both hit-rate gains came from the same move: tightening what counts as evidence for entering protected stash space. v001 removed the blind path and v002 removed stale evidence from the selective path. Neither added protection. This is the fourth cycle in a row where the zero-sum rule for protected capacity has held.
- Recency matters more than the amount of evidence. In 0006 a larger ghost window lost. This cycle a gate at roughly half a window won. The best ghost horizon looks shorter than one cache's worth of evictions, and direct-mapped overwrite alone does not age entries out quickly enough.
- BIP's optimum looks like "off". On this trace, the reuse it caught beyond the ghost's reach cost more capacity than it returned. Removing it also cuts metadata and per-insert work.
- v001 and v002 are orthogonal (one changes the blind path, the other the selective path), and both were positive on the same parent. They have not been combined yet.
- The cohort's one variant aimed at efficiency produced no signal. An exactly identical result means either the cascade branch never fires at this size or the change did not apply. Per-op-time still has no fitness dimension, so this kind of hypothesis cannot be scored yet.
- For the fifth cycle running, writers could not inspect the overlay source. Every mechanism claim is taken from the hypothesis text and has not been checked against the diff. Evaluation is still one LIRS trace at one size.

## Open questions

- Do v001 and v002 stack (gated ghost with no BIP), or do they reclaim the same protected slots?
- Where is the best gate cutoff: tighter than half the window, or looser? Can the ghost table shrink to fit the gated horizon and save metadata?
- Does the cascade path ever fire at this cache size? A displacement counter would settle whether H03 was a no-op.
- Does removing BIP hurt on traces where useful reuse falls outside the ghost horizon?

## Continuation

Build on v002 (recency-gated ghost). Combine it with v001's BIP removal, then sweep the epoch cutoff and try shrinking the ghost table to the gated horizon. Drop larger ghost windows, extra ghost-hit protection, and any change that adds protection rather than filtering it. Stop proposing per-op-time hypotheses like cascade-free routing until the eval reports per-op-time, metadata-size and a displacement counter. Push again for a second trace or cache size before more micro-tuning.
