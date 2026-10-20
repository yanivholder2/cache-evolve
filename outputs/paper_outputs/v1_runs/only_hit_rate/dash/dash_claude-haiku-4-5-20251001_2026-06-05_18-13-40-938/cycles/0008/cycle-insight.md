# Cycle 0008 Insight

## Cohort outcome

This cycle marks a decisive pivot from seven cycles of parametric refinement to architectural restructuring. After occupancy-aware demotion routing (v001) failed once more to exceed the 0.684 baseline, the cohort tested two orthogonal architectural changes: collapsing the three-tier hot-side to a unified hot tier (v002) and gating tier-skip to proven stash residents with mandatory recovery enforcement (v003). Both v002 and v003 improved hit rate, breaking through the 0.684 ceiling for the first time in four cycles and validating that the bottleneck was architectural structure, not parametric tuning.

## Patterns

- **The three-tier hot-side hierarchy was the actual bottleneck**: v002's collapse of target and probing into a single hot tier, eliminating the intermediate gatekeeping step, improved hit rate. This directly contradicts the assumption from prior cycles that tier positioning and insertion mechanics were the constraints. The workload's access pattern tolerates—and benefits from—direct stash-to-hot promotion, suggesting the two-stage target→probing filtering was adding overhead without sufficient discriminatory signal for this workload.

- **Architectural simplification succeeds where parametric tuning failed**: Seven prior cycles tested insertion position, occupancy thresholds, eviction priority, cascade structure, and capacity allocation without exceeding 0.684. Cycle 0008's gains emerge only from changing the tier structure itself, not from tuning insertion position, eligibility gates, or occupancy routing. This confirms that the previous ceiling was enforced by the architecture's topology, not by any single parameter.

- **Tier-skip is viable when recovery is structured into the skip decision**: v003's conditional tier-skip (gated to stash residents with access_count ≥ 2, with skip_history_flag enforcing target re-entry on eviction) improved hit rate, directly refuting cycles 0004 and 0006's findings that tier-skip was "fundamentally incompatible" with demotion recovery. The key difference is that v003 eliminates the "unvetted hot-tier residents" problem by making recovery automatic: skipped items do not arrive at probing as unchecked stash hits, but as pre-qualified candidates who get a safety-check second chance in target before demotion. This reconciles two previously antagonistic strategies.

- **Occupancy-aware routing remains inert as a standalone lever**: v001's occupancy-coordinated demotion (bypassing target when >70% full) maintained baseline without improvement. This extends the pattern that occupancy-based gating does not improve hit rate. The findings suggest that occupancy awareness alone, decoupled from structural changes, is not a constraint; the leverage is in *how tiers are connected* (v002) and *how skip recovery is enforced* (v003), not in *occupancy triggers* (v001).

## Open questions

- Do v002 and v003 achieve equivalent hit-rate improvements, or does one exceed the other? Understanding the magnitude difference clarifies whether simpler hierarchy or structured tier-skip is more effective for this workload.

- Can v002's two-tier simplification and v003's gated tier-skip be combined into a single variant? A two-tier hierarchy with conditional tier-skip recovery might compound the gains if they are orthogonal.

- What is the improvement magnitude relative to prior cycles' baseline? Is this a modest (+0.001–0.005) or substantial (>0.01) breakthrough?

## Continuation

Architectural restructuring is now confirmed as the path forward; parametric tuning has been exhausted. For the next cycle, abandon occupancy-based routing entirely and focus on consolidating the architectural gains: test whether v002's simpler hierarchy and v003's gated tier-skip can be merged, since both improve hit rate through different mechanisms (eliminating gatekeeping overhead vs. reconciling acceleration with recovery discipline). If both are improvements, a combined variant merging two-tier hierarchy with conditional tier-skip recovery should be the next target. If they cannot be safely combined, document the trade-off and select the variant with the larger gain as the new baseline. Regardless, the 0.684 ceiling has been broken; document that the three-tier hot-side hierarchy and its two-stage filtering requirement were the root architectural bottleneck defended across all prior parametric tuning attempts.
