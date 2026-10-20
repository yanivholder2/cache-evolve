# Cycle 0010 Insight

## Cohort outcome

Three hypotheses probed the two live design branches in parallel: value-split decay on the
single-queue stack (v001), a 25/75 SLRU segment re-split (v002), and a post-decay cold
re-scan in the warm-lock path (v003). None improved hit-rate materially; v001 was selected
as winner by virtue of being exactly neutral rather than a regression.

## Patterns

- **The counter gradient above the lukewarm tier carries no actionable eviction signal.**
  Preserving hot items at counter 2–3 rather than 1–2 after decay made no difference:
  the pre-decay cold-first scan already shields hot items from eviction, so the frequency
  signal conserved by the asymmetric decay never changes which item gets removed. Both
  the position-split (cycle 0009) and value-split (this cycle) decay variants are now
  confirmed inert, closing the decay-asymmetry axis entirely.
- **SLRU segment ratio is a near-zero lever on workload-a.** The 25/75 split yielded a
  small positive nudge but not a meaningful gain; the offsetting shrinkage of the
  probationary buffer erased most of the benefit from the larger protected tier. The
  ratio knob is exhausted alongside the SLRU family more broadly.
- **Post-decay cold re-scan recapitulates the cycle-0007 lesson.** Selecting a
  counter-0 interior item instead of the LRU front discards temporal locality — the
  mid-array item is typically fresher, not staler, than index 0. Index 0 remains the
  correct warm-lock eviction target on this layout.
- **Three consecutive cycles of null or negative results signal a hard local optimum.**
  Cycles 0008–0010 have together closed asymmetric decay, SLRU segment geometry, and
  post-decay re-scans. The single-queue accumulated stack is not improving; the SLRU
  branch never surpassed it. The hit-rate frontier appears saturated for workload-a.

## Open questions

- Per-op latency has been deferred since cycle 0005 and remains the sole unmeasured
  dimension. Is the single-queue stack's hit-rate advantage worth its mechanism
  complexity versus a simpler design?
- Is workload-a itself the binding constraint? A trace with stronger post-eviction
  temporal locality or a heavier scan component might rank the candidate designs
  differently.

## Continuation

The hit-rate story is conclusive: the single-queue accumulated stack dominates every
challenger on workload-a, and no parametric or structural variant within the current
design family has moved the needle across ten cycles. The single actionable next step
is the long-deferred latency audit — measure per-op cost for the accumulated stack
versus the best SLRU combination (cycle 0009 v003) to determine whether the hit-rate
lead also comes with a latency advantage, or whether a simpler design is competitive
on efficiency. If the single-queue stack wins on latency too, declare it the final
design for workload-a and pivot to either a new workload or a qualitatively different
policy family. Do not revisit decay asymmetry, SLRU segment ratios, post-decay cold
re-scans, or any other axis already confirmed closed.
