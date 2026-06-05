# Cycle 0009 Insight

## Cohort outcome

This cycle ran the last untested SLRU placement lever (H01: re-admit
ghost keys at the protected-MRU/warmest end), the named ARC-tuning thread
from cycle 0008 (H02: clamp ARC's adaptation target `p` to ~5% to mimic
the 95/5 split), and a brand-new 2Q skeleton off baseline (H03: FIFO
admission + ghost-driven promotion). v001 took the Winners slot only
because it is anchored to the incumbent and regressed least in absolute
terms — another win-by-confirming-a-negative. Nothing beat the 95/5 SLRU
incumbent.

## Patterns

- The warm-end re-admission placement is disconfirmed, closing the last
  open SLRU axis. At 95/5 the protected segment *is* essentially the whole
  cache, so injecting single-reuse ghost keys at the protected MRU dilutes
  the hot set and forces tenured keys down through the already-binding
  probationary floor — fast post-eviction reuse is too weak a signal to
  earn maximum protection when protected room is the scarce resource.
- The ARC-tuning thread is ruled out as a no-op. Capping `p` near 5% did
  not reproduce the incumbent's lift because `p` never climbs above the
  ceiling on this reuse-heavy trace, so the clamp never bites. ARC's gap
  to the incumbent is therefore *not* an over-large recency tier; it lives
  in T2's internal ordering/admission, not its size.
- 2Q is the third structural skeleton (after LIRS and ARC) to beat the
  single-LRU baseline — FIFO admission filtering one-hit-wonders before
  they reach the main list is a genuine lift. But, exactly like LIRS and
  ARC before it, that lift was measured against baseline and never raced
  directly against the 95/5 incumbent.

## Open questions

- The recurring blind spot persists: each new skeleton (LIRS, ARC, 2Q) is
  parented on baseline and reports a baseline-relative lift, yet no clean
  head-to-head against the 95/5 SLRU incumbent has ever been run. Does
  *any* of these structures actually clear the incumbent, or are they all
  below it / within noise?
- With ARC's size lever ruled out, is ARC's residual gap tunable via T2
  ordering/admission, or is that skeleton simply below the incumbent?

## Continuation

Drop both exhausted threads: SLRU warm-end re-admission is disconfirmed
and ARC `p`-sizing is a confirmed no-op — do not reopen either. The live
structural thread is now 2Q, the latest skeleton to beat baseline. But
the binding unfinished business for three-plus cycles is the direct
comparison no cohort has executed: next cycle should race 2Q (and, if
cheap, ARC) *directly against the 95/5 SLRU incumbent on its home trace*,
observable contract verified, rather than spawning yet another
baseline-parented skeleton. If 2Q trails the incumbent like LIRS and ARC
did, declare the cohort strongly converged on the 95/5 SLRU optimum.
