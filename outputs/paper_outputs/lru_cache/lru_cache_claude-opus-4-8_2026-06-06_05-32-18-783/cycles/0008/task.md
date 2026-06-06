# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0008

## Winners

- v003
- v001
- v002

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0008/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0008/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0008/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0008/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle attempted to cash the Cycle 0004 follow-ups: fold the gated
ghost admission onto the 24-byte node (v001), tighten admission with a
two-hit probation gate (v002), and fuse the promote+demote pair to cut
hot-path churn (v003). All three held the 24-byte node class and all
three landed net neutral-to-negative on their bases — none advanced the
frontier. The standing champion (Cycle 0004's 24-byte node) was not
displaced.

## Patterns

- Every "free mechanism" claim held but every policy it encoded moved
  the wrong way. Spare-high-bit flags (`REF_MASK`/`PROT_MASK`), the
  in-struct ghost store, and pure control-flow refactors each added zero
  node bytes — yet the gated ghost fold, the two-hit gate, and the fusion
  all regressed latency and/or hit-rate. The size mechanism is proven
  cheap; the behavioral lever is what failed.
- Workload-a's reuse is short-distance, and that drains the admission
  levers. Ghost admission fired too rarely to move recall; the second-hit
  gate delayed promotion of genuinely-hot keys and pushed hit-rate below
  parent. The hit-rate headroom that admission cleverness opened in
  Cycle 0004 did not reproduce here — it looks trace/size-contingent, not
  a general win.
- "Fusion" of two correct list-surgery calls is not redundancy removal:
  the demote-on-overflow path already does exactly four surgeries, so
  collapsing the calls performs the same work plus a hot-path branch and
  a parallel code path — regressing even the untouched `put`, the tell of
  codegen perturbation rather than algorithmic saving.

## Open questions

- Is the Cycle 0004 gated-ghost hit-rate gain actually realizable on
  workload-a, or was it specific to the trace/size it was measured on?
  v001's near-zero recall delta is a direct warning that the fold's
  premise may not hold here.
- With admission gating now tapped out on this workload, where does
  remaining headroom live — back on the latency/layout axis, or is the
  current champion close to this trace's ceiling?

## Continuation

Drop the v001 ghost-onto-24-byte fold: it is structurally per-op-costly
while its benefit is absent on workload-a — the cross-cycle additive
prediction did not hold. Drop two-hit admission gating and the
promote/demote fusion as tried-and-regressed. Stop chasing hit-rate via
admission cleverness on this workload; before any further policy work,
the next cycle should verify whether the Cycle 0004 hit-rate record even
reproduces on workload-a, and otherwise pivot back to latency/layout
levers. Cycle 0004's 24-byte node remains the standing base.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle ran two latency micro-optimizations against the 24-byte champion
and one geometry-sizing probe. Both micro-opts — eliding arena bounds
checks via unchecked indexing (v001) and fusing the unlink+push_back
relink to drop "dead" sentinel writes (v002) — held hit-rate and node size
but regressed per-op latency, advancing nothing. The lone advance was v003:
nudging the protected soft cap from 4/5 to 7/8 of capacity recovered
hit-rate above the parent with no latency penalty.

## Patterns

- Both latency micro-opts failed for the same reason and it is a cost-model
  error: the "redundant work" each removed was already free. Bounds checks
  on sequentially-predicted, proven-in-bounds arena accesses were elided or
  cheap (v001); the eliminated NIL stores were store-buffer-absorbed writes
  to an already-resident cache line (v002). In both cases the change traded
  a non-cost for real codegen damage — unsafe risk and register/code-size
  pressure from inlining — and came out net slower. The hot-path latency of
  this SLRU lives in hashing, link surgery, and cache-line traversal, not in
  the checks or writes these variants targeted.
- The Cycle 0004 lesson that protected-cap widening costs latency is now
  shown to be magnitude-dependent, not categorical. v003's modest one-eighth
  step bought hit-rate for free — get/put p50 even ticked down — whereas
  v003-from-Cycle-0004's larger share starved the fast path. There is real
  headroom between the parent's 4/5 geometry and the point where latency
  breaks down.

## Open questions

- How far can the protected fraction be pushed past 7/8 before the
  fast-path starvation seen at larger shares reappears — is there a clean
  cap maximizing hit-rate while latency holds?
- With the obvious micro-opt levers tapped, is the 27 ns champion latency
  near this trace's floor, or does a structural change (hashing, traversal
  layout) still have room the redundant-work edits never reached?

## Continuation

Promote v003's 7/8 protected cap as the new base — it is a free hit-rate
gain and the only frontier mover this cycle. Next cycle, probe the
remaining geometry headroom (a fraction between 7/8 and the starvation
point) rather than reaching straight for the largest share. Drop
bounds-check elision and the relink fusion as tried-and-regressed: both
attacked non-bottlenecks. If pursuing latency further, target the actual
hot-path costs (hashing, link surgery, cache-line traversal), not
compiler-handled redundancy.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle probed the headroom around the new 7/8 protected-cap champion
from three angles: a larger geometry step to 15/16 (v001), a stricter
two-hit probation gate (v002), and a last-key get cache to skip the
HashMap probe (v003). Both surviving Variants held the 24-byte node but
regressed their bases — neither advanced the frontier. The lone latency
probe (v003) never produced a measurement, lost to a simulator infra
failure rather than on merit.

## Patterns

- The protected-cap headroom between 7/8 and the starvation cliff is now
  closed from the top. Cycle 0006 found free hit-rate at 7/8; stepping to
  15/16 confirmed a sharp starvation cliff in that gap — the near-full
  protected segment leaves the fast path no room, so get/put p50 and p99
  all jumped for a small hit-rate gain. 7/8 sits just below the breakdown
  point, and there is little usable geometry left above it.
- Admission cleverness keeps failing for the same workload reason. As in
  Cycle 0005's two-hit gate, the two-hit warm filter drains short-reuse
  keys out of probation before their second hit lands, so it discards
  genuine repeaters and drops hit-rate — while adding per-hit bookkeeping
  that nudges get p50 up. The free-bit mechanism worked; the policy is
  the recurring loser on this short-reuse trace.
- The mechanism-vs-policy split holds yet again: 15/16 sizing and the
  warm-bit packing both kept the 24-byte layout byte-for-byte; every loss
  came from behavior (geometry starvation, admission policy), not from the
  representation changes.

## Open questions

- With both the top of the geometry headroom and admission gating now
  exhausted, is the 7/8 champion effectively at this trace's
  hit-rate/latency frontier?
- Does the H03 last-key cache — the only attempt that actually targeted
  the named get hot-path cost (HashMap hash+probe) — have merit? It was
  never evaluated, so the latency-via-traversal/hashing thread remains
  genuinely untested, not refuted.

## Continuation

Keep Cycle 0006's 7/8 protected cap as the standing base; nothing this
cycle displaced it. Drop the 15/16 geometry step (starvation cliff
confirmed) and the two-hit warm gate (admission gating is tried-and-
regressed on this short-reuse workload — stop revisiting it). The one
live thread worth re-running is H03's last-key get cache: it attacks the
real hot-path cost and died on infra, not measurement, so re-issue it
before concluding the champion is at the frontier. If pursuing latency
further, stay on actual hot-path costs (hashing, link surgery, cache-line
traversal), not geometry or admission policy.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
