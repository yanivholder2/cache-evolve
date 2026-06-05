# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0009

## Winners

- v001

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0009/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0009/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0009/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0009/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle followed up cycle 0005's two live threads: tune the LIRS
rebuild by transplanting SLRU lessons onto it (H01 widen the resident-HIR
window, H02 route re-admitted ghost hits to the weak tier), and close the
last SLRU geometry question by pushing the protected ratio to 97/3 (H03).
All three regressed. v003 (the 97/3 probe) won only as the
least-regressive change, and it did so by confirming a negative.

## Patterns

- The SLRU geometry axis is now formally closed at 95/5. Pushing to 97/3
  shrank the probationary reuse window below the floor needed to catch
  second touches, and the marginal protected slots were worth less than
  the drain capacity they consumed — the binding constraint is the
  probationary floor's *size*, exactly as prior cycles diagnosed.
- SLRU tuning lessons do not transplant onto LIRS. Both LIRS levers
  failed for structural reasons: the HIR window and LIR budget are
  zero-sum, so widening the reuse-detection queue directly evicts the
  low-IRR blocks LIRS was protecting; and a non-resident-HIR reuse is
  itself the canonical LIR-qualifying signal, so demoting it to the weak
  tier (the SLRU ghost-placement win) withholds protection LIRS correctly
  grants. LIRS's own S-membership and stack pruning already do the
  filtering SLRU bolted on after the fact.
- The cross-policy transfer hypothesis is disconfirmed: lessons learned
  inside one skeleton are not portable as parameter edits to another with
  a different invariant structure.

## Open questions

- Cycle 0005's central question — is v002's reported LIRS lift real and
  does it beat the 95/5 SLRU incumbent? — remains unresolved. This cycle
  only tried to *tune* LIRS (and failed both tunings); it never cleanly
  re-ran the untuned LIRS parent against the incumbent to confirm or
  refute the lift.
- With both LIRS tuning levers regressing from the parent config, is the
  LIRS parent at its own local optimum, or is there an untested lever
  (LIR/HIR warm-up, non-resident bound) that actually moves it?

## Continuation

Treat the SLRU line as fully converged at 95/5 — do not probe geometry
again. The one unfinished piece of business is the LIRS verification
cycle 0005 called for and this cycle skipped: next cycle should cleanly
re-run the untuned LIRS parent, confirm whether its hit-rate lift is real
and beats the 95/5 SLRU incumbent on its home-turf trace, and verify the
observable contract. Do not attempt further LIRS tuning by transplanting
SLRU parameters — that transfer is disconfirmed; tune LIRS only against
its own measured behavior. If the untuned LIRS lift does not survive a
clean re-run, declare the cohort converged on the 95/5 SLRU optimum.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
