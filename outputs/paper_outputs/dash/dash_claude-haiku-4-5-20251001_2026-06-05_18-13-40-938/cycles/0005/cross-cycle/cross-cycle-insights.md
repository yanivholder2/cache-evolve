# Cross-Cycle Insights

## Recurring wins

- **Stash-hit acceleration via deterministic tier-skipping**: Cycles 0001–0002 both achieved measurable hit-rate improvement by fast-tracking items proven to reach the stash directly toward higher tiers, validating recency-at-survivorship as a robust promotion signal.

- **MRU back insertion for demoted items**: Cycle 0003 winner that persisted unchallenged through cycles 0004–0005; demoted items benefit from immediate MRU positioning upon reinsertion, enabling faster re-qualification for advancement. This is the only mechanism that survived combination testing intact.

- **Deterministic positioning rules outperform conditional logic**: Cycles 0002–0005 consistently showed that simple, uniform insertion rules (always back for demoted items) beat occupancy-adaptive or load-aware destination selection, indicating the system rewards predictability over state-aware branching.

- **Hotness signals gate acceleration more effectively than occupancy**: Strategies gating promotion on recency/fitness (stash hit, tier advancement history) succeeded across cycles; those relying on occupancy thresholds alone repeatedly failed or suppressed hit rate.

## Recurring failures

- **Frequency-based eviction (LFU)**: Cycle 0001 test showed no gain; recency dominates this workload.

- **Occupancy-driven filtering without fitness signals**: Cycles 0002, 0003, and 0005 show occupancy thresholds alone suppress or leave hit rate unchanged; occupancy is not a proxy for hotness.

- **Combining previously successful strategies**: Cycle 0004 demonstrated that stash-hit skip, demotion back insertion, and fresh-promotion MRU create interference when stacked; the system has reached an interaction boundary.

- **MRU positioning for fresh promotion paths**: Cycle 0004 exposed asymmetry—demoted items benefit from MRU (cycle 0003), but newly promoted items in higher tiers are destabilized by it.

- **Intra-tier optimization and eligibility gating**: Cycles 0004–0005 showed that within-tier repositioning and promotion-candidacy delays are orthogonal to hit-rate improvement, leaving the underlying bottleneck untouched.

## Open questions

- **Is 0.684 a true local optimum, or is architectural headroom available?** Cycles 0004–0005 tested three distinct levers without breaking through, suggesting the ceiling may be enforced by tier capacity ratios, eviction cadence, or bucket saturation rather than positioning knobs.

- **What architectural constraint enforces the hit-rate plateau?** The robust defense across orthogonal approaches hints at a constraint in tier occupancy balance, stash depth, or eviction policy interactions not yet exposed for testing.

- **Why does the same insertion mechanism benefit demoted items but harm fresh promotion?** This asymmetry (cycles 0004–0005) may be fundamental to tier interaction or point to a tuning opportunity; clarifying it could unlock gains or confirm the equilibrium is optimal.

- **Should the next cycle pivot to tier architecture or exhaust insertion-mechanics variants?** If occupancy, positioning, and timing knobs are spent, testing tier capacity ratios, alternative eviction orderings, or stash saturation thresholds may reveal new levers.

## Per-policy notes

- **Stash-hit acceleration (tier-skip)**: Won early (cycles 0001–0002) by filtering items late in the hierarchy, but conflicts with demotion back insertion when combined (cycle 0004); these mechanisms assume incompatible filtering disciplines.

- **MRU positioning strategies**: Striking asymmetry—highly effective for demoted-item recovery (cycle 0003 winner, stable through 0005), neutral or harmful for fresh promotion paths. Within-tier MRU reordering is orthogonal to hit-rate gains.

- **Occupancy-adaptive thresholds**: Failed or plateaued across four distinct tests (cycles 0002, 0003, 0004, 0005). Occupancy without fitness signals acts as a suppression valve; this policy class appears exhausted.

- **Frequency-based policies (LFU)**: Single test (cycle 0001) showed no gain; recency-based eviction is superior for this workload. Not pursued further.
