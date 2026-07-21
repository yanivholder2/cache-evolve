# Cycle 0006 Insight

## Cohort outcome

This cycle tested three compositions built on the link-array structural baseline: a standalone quota enlargement (v001), quota enlargement paired with SEEN_BIT two-hit admission (v002), and SEEN_BIT with an eviction-path second-chance rescue (v003). Only the pure quota change won; both SEEN_BIT variants hurt hit-rate.

## Patterns

- **Quota enlargement is validated and cost-free.** v001 cleanly reproduced the hit-rate improvement first seen in Cycle 0005/v003 and confirmed that the apparent latency cost in that earlier run was measurement noise from an anomalous zero-floor baseline. The 87.5% protected share is now a settled, noise-free result.
- **SEEN_BIT's two-hit gate starves the policy it is meant to serve.** In v002, enlarging the protected pool offered no benefit because the stricter admission barrier prevented enough items from graduating to fill it. The mechanism's precondition — sufficient repeat accesses within the probationary window — is not met by this workload's access pattern.
- **Eviction-path rescue is undermined by SEEN_BIT reset.** v003's `push_back_prob` call unconditionally cleared SEEN_BIT, raising the effective graduation barrier from one additional hit to two for every rescued item. The rescue compounded the gate's over-strictness rather than softening it, making the displaced items net-more-valuable than the saved ones.
- **All two-hit mechanisms have now failed across three cycles.** Cycle 0004 (SEEN_BIT splice), Cycle 0005 (cost-free bit-OR), and Cycle 0006 (composition and rescue) each exposed the same root cause: workload-a does not generate enough within-window repeat accesses for a two-hit requirement to be net beneficial.

## Open questions

- Is there a softer, position-based or frequency-sketch-based admission variant that loosens the gate without fully abandoning selective graduation?
- With the quota now at 87.5% protected, is there additional headroom by pushing it further, or does the probationary pool's minimum useful size create a floor?
- Could directly measuring the probationary graduation rate (fraction of items reaching a second hit before eviction) guide the choice of admission threshold, rather than inferring it from aggregate hit-rate?

## Continuation

The next cycle should adopt v001's quota policy as the new structural baseline and retire SEEN_BIT entirely — three cycles of failure on this workload is conclusive. Two directions worth probing: (1) experiment with quota values beyond 87.5% to locate the point of diminishing returns or reversal; (2) explore a lighter admission filter, such as a compact frequency sketch that graduates any item with an estimated frequency above a low threshold, rather than requiring two sequential hits within a narrow window. Do not revisit the eviction-path rescue or any mechanism that requires a per-item state bit tracking the number of probationary hits seen.
