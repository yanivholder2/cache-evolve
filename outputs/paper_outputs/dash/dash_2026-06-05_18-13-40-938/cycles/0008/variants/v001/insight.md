# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Occupancy-aware demotion routing (bypassing target when it exceeds 70% occupancy) did not improve hit-rate—it remained flat at 0.684. This suggests that the decision to bypass or retry an intermediate tier based on occupancy alone is not the constraint limiting cache performance. The 0.684 ceiling persists despite three distinct eviction-routing interventions (H02's eligibility gate, prior H01's target-first proactive eviction, and this cycle's H01's occupancy-bypass logic), indicating the bottleneck is not in *where* evicted items flow or *when* they become eligible, but in the fundamental tier structure or the upstream promotion pathway itself.
