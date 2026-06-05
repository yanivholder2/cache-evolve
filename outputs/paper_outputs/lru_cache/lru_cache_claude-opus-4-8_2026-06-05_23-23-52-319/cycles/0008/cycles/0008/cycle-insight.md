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
