# Cycle 0004 Insight

## Cohort outcome

This cycle attempted three refinements on top of v003's vacancy-gated frequency promotion: a stricter ≥ 3 frequency threshold, a vacancy-gated stash bypass, and an lfu_counter reset at stash-to-target promotion. No variant moved any measured dimension positively — every regression was either structural (overlay coverage gap silently dropping the parent's Item-struct compaction) or workload-dependent (the target scenarios simply do not occur often enough on this trace).

## Patterns

- **Overlay coverage gaps are a first-class failure mode.** Two of three variants touched only `dash_segment.rs` and silently reverted the parent's `item.rs` struct reduction, re-inflating per-item footprint and worsening tail latency. The regression did not come from the hypothesis under test — it came from incomplete inheritance of the parent's accumulated changes. Any multi-file parent change must be fully re-applied in descendant overlays, or the measurement conflates intent with omission.
- **The vacancy gate's ≥ 2 threshold appears close to optimal for this workload.** Raising to ≥ 3 produced a fractional hit-rate loss rather than a gain, indicating that a real cohort of genuinely reusable items sits between the two thresholds. The prior cycle's threshold-only (no vacancy guard) ≥ 3 result had a different base, so the comparison was confounded; on the clean vacancy-gated base, stricter filtering is net harmful.
- **The joint stash-bypass condition fires too rarely to be measurable.** Requiring both target-full and probing-vacant simultaneously at stash-hit time is apparently an uncommon joint event on this trace, making the bypass mechanism inert. The hypothesis was structurally sound but the workload does not generate the precondition frequently enough.
- **Counter-carrying oscillation is not a live pattern in this workload.** The lfu_counter reset was designed to eliminate premature re-promotion of stash-re-entrants with stale counts, but no hit-rate signal emerged, implying the oscillation scenario is too rare on workload-a to be detectable at this cache size.

## Open questions

- Is there a mechanism to verify at hypothesis-generation time that a child overlay carries forward all files modified by its parent, preventing silent structural regressions from coverage gaps?
- At what cache-to-working-set ratio does stash traffic increase enough that the target-full + probing-vacant joint condition fires at a detectable rate?
- If the ≥ 2 threshold is near-optimal, are there other promotion axes (e.g., recency combined with frequency, or a decay on the counter over time) that could improve selectivity without the crude threshold knob?

## Continuation

Next cycle should not further tune the frequency threshold — the ≥ 2 vacancy-gated gate from Cycle 3's v003 appears to be the local optimum for this workload. The overlay-coverage-gap finding is the most actionable output of this cycle: future hypotheses that descend from multi-file parents must explicitly enumerate all inherited file changes, not just the files the new hypothesis modifies. Rather than continuing to refine promotion sensitivity, the next cycle should explore an orthogonal axis — for example, eviction ordering within a tier (which item gets displaced when a full bucket must eject), or the promotion path for items accessed in the probing tier rather than the target tier. The vacancy-gate and MRU-end insertion are now stable anchors; the remaining headroom is likely in eviction selection rather than promotion gating.
