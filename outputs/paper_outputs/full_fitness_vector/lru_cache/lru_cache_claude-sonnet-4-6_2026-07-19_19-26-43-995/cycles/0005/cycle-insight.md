# Cycle 0005 Insight

## Cohort outcome

This cycle executed exactly the two structural directions recommended by Cycle 0004: a Node layout split separating link metadata from key/value data (v002) and a cost-free two-hit admission filter using a SEEN_BIT (v001), plus a protected-quota enlargement (v003). Both structural variants landed as Winners; the quota-only change did not, blocked by an apparent (likely noise) per-op-time regression.

## Patterns

- **Winners won by not regressing, not by visibly improving.** Both v001 and v002 produced no measurable per-op-time hurt — v001 confirmed that bit-OR on the first probationary access is essentially free, and v002 confirmed that the link-array split is cost-neutral — yet neither showed a positive latency signal. The tight per-op budget established in prior cycles means the bar for winning is "do no harm" when measuring at current timer resolution.
- **Cost-free deferred graduation is structurally solved but policy-tuning is unfinished.** Reducing the SEEN_BIT first-access path to a single bit-OR resolved the Cycle 0004 open question on implementation cost, but the stricter two-hit admission policy exposes a workload sensitivity: enough single-access items exist that a meaningful fraction are evicted before reaching the second hit needed to graduate, hurting hit-rate. The mechanism is correct; the calibration is not yet right for this workload.
- **The link-array split achieved its layout goal without side effects.** Node size dropped from 24 to 16 bytes and link-traversal density increased substantially. The latency benefit is real in principle but falls below the current measurement floor — the absence of regression is still a clean endorsement of the structural direction.
- **The quota enlargement is directionally correct but measurement-opaque.** v003's hit-rate improvement is genuine; its apparent per-op-time hurt is most plausibly measurement noise from the parent's anomalously-zero p50 floor. The policy is still unconfirmed as cost-free because the noise masks the verdict.

## Open questions

- Can the SEEN_BIT two-hit filter (v001) and the enlarged protected quota (v003's direction) be composed? A deeper protected pool would keep entries that finally graduate resident longer, potentially recovering the hit-rate lost to stricter admission.
- Does the quota enlargement's apparent per-op regression persist or resolve when retested on the link-array baseline (v002), which now has a non-zero p50 floor to measure against?
- Is there a workload- or frequency-adaptive threshold that lets the SEEN_BIT filter skip two-hit enforcement for items it can infer will not recur, rather than applying the stricter policy uniformly?

## Continuation

The next cycle should compose the SEEN_BIT deferred graduation (v001) with the quota enlargement (v003's direction), built on top of the link-array structural baseline (v002). The hypothesis is that a larger protected pool compensates for the stricter admission gate: items that survive to a second hit are then shielded from demotion long enough to accumulate further hits. Additionally, the quota enlargement should be retested cleanly on the v002 base to determine whether its per-op-time signal is genuine or an artifact of the prior measurement floor. Do not revisit standalone structural layouts or adaptive policy mechanisms — the search space is now narrow enough to pursue targeted composition.
