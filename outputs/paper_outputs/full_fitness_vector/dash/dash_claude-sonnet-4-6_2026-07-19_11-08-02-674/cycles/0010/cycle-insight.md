# Cycle 0010 Insight

## Cohort outcome

This cycle probed three targeted adjustments: lowering the CLOCK_BIT warm-set threshold from freq≥3 to freq≥2 (v001), extending the two-chance clock pre-pass symmetrically to the probing tier (v002), and lowering the probing re-entry counter gate from ≥2 to ≥1 (v003). All three were named winners by elimination rather than by improvement — none produced a clear hit-rate gain, and two introduced hit-rate regressions, while individual latency improvements (v001 on put-path tail, v002 on put-path tail) were offset by hit-rate costs.

## Patterns

- **The CLOCK_BIT threshold search space is now exhausted by elimination.** freq≥1 saturated the warm set (Cycle 8); freq≥2 over-extended it because Zipfian access distributions concentrate heavily at exactly two accesses, reproducing the saturation problem in milder form (v001); freq≥3 remains the least-bad discriminator despite hot-bucket rotation overhead (Cycle 9). Simple arithmetic thresholds on the raw counter cannot simultaneously protect the Zipfian working set and maintain clock discriminating power — a structural change (rotation cap, density-adaptive gate, or independent bit) is needed.
- **Clock protection in the probing tier creates a hit-rate/latency trade-off rather than a win.** v002's probing clock pre-pass cuts put-path tail latency by front-loading cold-victim selection, but it stickifies warm probing residents — items that have earned CLOCK_BIT in probing are blocked from timely demotion, slowing the target→probing→target circulation that drives steady-state hit rate on this workload. The mechanism is directionally useful for latency but structurally antagonistic to promotion throughput unless gated by bucket occupancy.
- **The probing re-entry counter threshold does real filtering work.** v003 confirmed that ≥1 is too aggressive: items recycled to target on their very first probing hit bounce before accumulating enough tenure to justify the slot, increasing churn and degrading hit rate and get-path tail latency simultaneously. The vacancy condition alone is not the binding constraint — the counter gate contributes independent signal on this trace.

## Open questions

- Can a consecutive-rotation cap on `clock_evict_prepare` (abort after k cleared bits and fall back to LFU-minimum) bound hot-bucket tail latency without sacrificing the freq≥3 threshold's discriminating power?
- Can the probing clock pre-pass (v002's mechanism) be gated on bucket occupancy — firing only when the probing bucket is above some fill threshold — to preserve the latency saving while avoiding the promotion-stickiness that hurts hit rate?
- Are there alternative warm-set signals — such as a decayed counter or a per-item "recently eviction-rescued" boolean — that could provide sparsity without depending on absolute frequency counts that cluster problematically on Zipfian traces?

## Continuation

The threshold-tuning axis for CLOCK_BIT is closed: all integer boundaries (≥1, ≥2, ≥3) have been tested and none dominates. Next cycle should move orthogonally — either introduce a rotation cap as a pure latency fix on top of the freq≥3 base, or replace the threshold criterion entirely with a mechanism that fires on eviction rescue rather than access count. The probing clock idea (v002) has genuine latency value and deserves a second test with occupancy-gating to suppress the admission-blocking effect; this is a narrowly-scoped change to `promote_to_probing`. Probing re-entry at ≥1 is falsified and should not be revisited.
