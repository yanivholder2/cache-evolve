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
