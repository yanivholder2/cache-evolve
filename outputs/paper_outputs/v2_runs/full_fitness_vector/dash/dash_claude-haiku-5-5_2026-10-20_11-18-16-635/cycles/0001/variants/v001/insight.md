# Variant v001 Insight

## Helped

- per-op-time: Once a probing hit is recovered into its true target, later hits to that key are found on the target check, which runs before the probing check in `promotion_get`, so they skip the extra probing-bucket lookup. The put-side timing movement is not explained by this change, since `put` does not go through `promotion_get`, so it may be measurement noise.

## Hurt

- hit-rate: Recovering a probing hit via `promote_to_target(..., InsertPosition::Back)` inserts the key at MRU of a target bucket that is often full, evicting its LRU resident into the demotion cascade, so keys that were stable in the baseline get displaced and the hit rate on this trace drops.

## Takeaway

Pulling probing residents back to their home bucket at MRU costs more in target displacement and cascade churn than it saves by keeping hot keys in place on this workload. The change only helps latency, and not enough to offset the hit-rate loss, so recovery into target needs a gentler insert position or a guard against displacing a hot target resident.
