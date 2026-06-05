# Cycle 0010 Insight

## Cohort outcome

This cycle finally ran the long-deferred head-to-head: race 2Q against the
95/5 SLRU incumbent on its home trace, decomposed into three distinct arms
— 2Q geometry matched to ~5% admission (H01/v001), 2Q immediate first-hit
promotion (H02/v002), and the incumbent-lineage hybrid that grafts a 2Q
admission front onto the 95% protected tier (H03/v003). None beat the
incumbent's 0.7313. v003, anchored to the incumbent lineage, took the
Winners slot as the least-regressive change — another win-by-confirming-a-
negative.

## Patterns

- 2Q's gap to the incumbent is **not** geometry and **not** promotion
  timing — the two candidate causes were isolated and both ruled out.
  Matching the 95/5 split (v001) closed most of the deficit but still
  trailed; collapsing the delayed ghost gate to immediate first-hit
  promotion (v002) actually *regressed*, because 2Q's FIFO-then-A1out path
  is doing real filtering work that demands a key prove reuse across a full
  A1in lifetime before earning a main-list slot. Geometry is necessary but
  not sufficient; the ghost gate is load-bearing.
- The incumbent-lineage hybrid confirms the same binding constraint from
  the opposite direction: single-hit promotion already pulls every
  re-touched key out of the floor, so swapping the floor LRU→FIFO is a
  no-op for un-promoted keys, leaving only A1out cross-lifetime
  readmission as a live mechanism — and promoting those keys straight to
  protected pollutes the tier while the readmission band stays too thin to
  pay for the protected incumbents it displaces.
- This re-confirms, now from the 2Q lineage, what reprieve and ghost arms
  found earlier: on workload-a the binding constraint is protected-tier
  *size*, and the warm-but-evicted band is too sparse to monetize. 2Q
  joins LIRS and ARC as a baseline-beating skeleton that does not clear
  the incumbent when raced directly.

## Open questions

- With LIRS, ARC, and now 2Q all raced (or partially raced) against the
  incumbent and all trailing, is there any structural skeleton left
  untested, or is the 95/5 SLRU optimum now demonstrably unbeatable on
  this trace?
- ARC's residual gap was never narrowed to a single mechanism the way 2Q's
  just was — its T2 ordering/admission remains the one unfinished
  decomposition, though appetite for reopening it is low.

## Continuation

The recurring blind spot is finally closed: the direct comparison that
cycles 0005–0009 kept deferring was run, and 2Q trails the incumbent like
LIRS and ARC before it, with its deficit decomposed and pinned to neither
geometry nor promotion timing. Drop all 2Q work — geometry, promotion, and
the admission-front hybrid are all disconfirmed. Three alternative
skeletons have now each beaten baseline yet failed the incumbent, and
every on-axis SLRU lever is exhausted. The next cycle should declare the
cohort strongly converged on the 95/5 SLRU optimum unless it can name a
genuinely untested structural axis — not another baseline-parented
skeleton destined to re-run the same baseline-relative comparison.
