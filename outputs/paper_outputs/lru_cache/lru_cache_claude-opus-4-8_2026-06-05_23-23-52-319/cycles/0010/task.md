# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0010

## Winners

- v003

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0010/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0010/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0010/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0010/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle attacked three distinct, previously-untested levers against
the converged 95/5 SLRU incumbent: a third recency-of-Nth-access "hot"
tier inside protected (H01), an adaptive ghost-fed boundary that floats
the protected/probationary split at runtime (H02), and a widened LIRS
non-resident horizon (H03). All three regressed. v002 (adaptive
boundary) landed in the Winners only as the least-regressive change,
winning by confirming a negative.

## Patterns

- The 95/5 split is not just the best *static* point — it resists being
  improved *adaptively*. Letting the boundary float on reuse-distance
  feedback drifted to a worse partition than the constant it replaced,
  because the two nudge signals fire asymmetrically and the up-nudges
  erode the very probationary floor that all prior cycles identified as
  the binding constraint. The cache cannot discover a better split from
  reuse feedback alone here.
- Stratifying protected by reuse *depth* (LRU-K-like, not frequency)
  also fails: a depth-split protected segment starves its cold tier and
  churns out the twice-seen working set before it earns a third hit. A
  single flat protected LRU retains the reuse-heavy set better — reuse
  depth is not the binding signal on this trace.
- LIRS still has no demonstrated headroom. Widening the non-resident
  horizon surfaces exactly the long-distance, high-IRR second touches
  that should stay unprotected, and the unconditional non-resident-to-LIR
  promotion turns the extra memory into LIR pollution. The harm is in
  admission policy, not horizon length, so the capacity-bound horizon was
  already at or past optimum.

## Open questions

- Every off-axis lever now tried — depth stratification, adaptive
  geometry, LIRS horizon — has regressed. Is there any remaining
  structural axis on this trace, or is the incumbent a true converged
  optimum?
- The long-standing question from cycles 0005-0006 — does an *untuned*
  LIRS parent actually beat the 95/5 SLRU incumbent on a clean re-run —
  remains unresolved; this cycle only tuned LIRS (and failed) rather than
  verifying the parent.

## Continuation

Treat SLRU geometry as definitively closed: static is exhausted at 95/5
and adaptive resizing is now also disconfirmed — do not probe boundary
tuning again. Depth-stratified protected tiers and widened LIRS
non-resident memory are both dead; drop them. The single piece of
unfinished business is still the clean untuned-LIRS-vs-incumbent
verification that cycles 0005-0006 called for and no cycle has executed.
The next cycle should either run exactly that verification or, if it
again declines, declare the cohort converged on the 95/5 SLRU optimum —
three consecutive cycles of off-axis probes have now all regressed.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle paired two more drain-floor probes with one genuinely new
skeleton: a single-lap reprieve for demoted protected keys inside the
95/5 SLRU frame (H01), a from-scratch ARC rebuild off plain baseline that
separates recency from frequency (H02), and a reactive resident-HIR floor
on canonical LIRS (H03). v001 (the reprieve) landed in the Winners by
sitting exactly on the incumbent — a confirmed neutral. The standout
thread: v002 (ARC) reported a real hit-rate lift over baseline yet did
**not** win, repeating the cycle-0005 pattern where a reported-gain
variant fails to take the cohort.

## Patterns

- The drain-floor axis is now exhausted in *both* skeletons. The SLRU
  reprieve almost never fires (the ~1945-slot protected tier rarely
  overflows at 95/5) and the few reprieved keys are too cold to earn a
  second touch in one more lap; the LIRS reactive floor demotes genuine
  low-IRR LIR blocks straight to the eviction front. Static window,
  ghost re-admission, and now reactive minimum have all regressed —
  protecting the drain queue cannot reach the incumbent.
- ARC is the first *alternative skeleton* to show a real structural lift
  over plain LRU. Separating recency (T1) from frequency (T2) with
  two-sided ghost-steered adaptation keeps hot keys resident through
  recency churn — the long-sought "exploitable off-axis structure." But
  that lift was measured against baseline, and the variant still did not
  beat the tuned 95/5 SLRU incumbent that won the cohort.
- The incumbent's static tuning continues to out-value structural
  adaptivity on this trace: cycle 0007 disconfirmed adaptive SLRU
  resizing, and now even ARC's balanced two-ghost adaptation does not
  clear the 95/5 bar despite a genuine gain over baseline.

## Open questions

- Does ARC's reported lift actually fall short of the 95/5 incumbent, or
  was it within noise / disqualified — exactly the unresolved question
  that dogged the LIRS parent across cycles 0005-0007? A clean
  ARC-vs-incumbent comparison has not been run.
- Is ARC's recency/frequency separation *tunable* (e.g., SLRU-style
  protected sizing on T2) to exceed the incumbent, or is the static 95/5
  optimum structurally unbeatable here?

## Continuation

Close the LIRS axis for good — static window, ghost diversion, widened
horizon, and reactive floor have all regressed; drop all LIRS work and
all drain-floor/reprieve tweaks inside SLRU. The live thread is now ARC,
the first skeleton to beat baseline structurally. Next cycle should run
the clean comparison this cohort skipped: untuned ARC directly against
the 95/5 SLRU incumbent on its home trace, with the observable contract
verified. If ARC trails, the incumbent is a strongly converged optimum;
if it is close, tune ARC's T2 sizing rather than reopening SLRU geometry.


### Cycle 0009

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
