# Variant v002 Insight

## Helped

- per-op-time: dropping the `is_hottest` gate removed the per-target-hit `promotion_candidate_index()` scan and the branching `else` LRU-bump path, so the get hot path became a straight remove-and-promote and both get and put p50/p99 fell.

## Hurt

- hit-rate: unconditionally promoting every re-accessed target item floods the probing bucket, so each warm key promoted now cascades/evicts another resident, shrinking effective retention instead of widening it — hit-rate dropped below baseline.

## Takeaway

The hottest-only gate was not dead weight: it filtered which warm keys earn probing, and removing it let aggressive promotion churn the probing tier and evict otherwise-retained keys, so the predicted hit-rate gain reversed into a loss. The only real win was incidental — skipping the hottest-candidate scan made every get cheaper.
