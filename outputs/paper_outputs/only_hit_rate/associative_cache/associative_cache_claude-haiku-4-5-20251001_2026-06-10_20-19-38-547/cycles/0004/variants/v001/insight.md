# Variant v001 Insight

## Helped

- **hit-rate**: Rebalancing bucket geometry to favor fewer, wider buckets reduces collision clustering by spreading items across larger buckets, lowering the rate of evictions that would otherwise flow to the stash.

## Hurt

(None measured in this evaluation)

## Takeaway

Wider bucket geometry provides measurable hit-rate improvement by attacking collisions at their source rather than relying solely on stash recovery. The 0.26% gain, while modest, confirms that reducing collision-driven evictions is an orthogonal improvement path to the 25% stash mechanism from v002. The synergy is real but narrow: the stash mechanism already captures most evicted items worth recovering, leaving limited margin for geometry changes to further compress the eviction stream. Wider buckets work best when collision pressure is the binding constraint, but workload-a suggests diminishing gains once stash recovery is already in place.

