# Variant v001 Insight

## Helped

(None measured in this evaluation)

## Hurt

- **hit-rate**: Reducing the reduction_factor from 0.71 to 0.70 pushes bucket geometry beyond the local optimum, trading optimal collision distribution for buckets that are too wide, degrading cache efficiency on this workload.

## Takeaway

The fine-grained local search hypothesis was incorrect: 0.70 represents a downward deviation from the true optimum, not a refinement toward it. The parent's 0.71 factor was closer to the optimal point in the collision-reduction search space than the tighter 0.70. This suggests the local optimum lies at or near 0.71, and further reduction crosses the boundary where bucket-width benefits saturate and begin to reverse. The 0.71–0.70 range appears to mark the edge of the stable efficiency region.
