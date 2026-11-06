# Variant v002 Insight

## Helped

- hit-rate: Every trace and size moved up a little, and the 16K configurations moved most. With the probing room gate, a hot target hit no longer evicts probing's LRU. Before, that evicted item was a proven resident, and it was pushed back to target or cascaded one of target's residents into stash. Now both tiers keep their contents, and the hot item stays protected at target MRU. Small caches gain more because full probing buckets are more common there, so the gate fires more often.

## Hurt

- per-op-time: `get` p50 and p99 did not move, so the hypothesis's main claim fails. `put` p99 regressed somewhat. The likely cause: hot items now stay in target instead of draining into probing, which keeps target buckets fuller. Puts and second stash hits then more often reach the "both full → target + demotion cascade" fallback that this change left untouched. In other words, the cascade cost moved from `get` to `put`.

## Takeaway

The target-hottest → probing cascade is not what keeps `get` p50 above baseline. Removing it from normal-tier hits left the median unchanged, so it is too rare on the median path to matter. The remaining p50 gap must come from the common hit path itself, such as the multi-bucket lookup order and the stash checks, not from cascades. The room gate is still worth keeping as a small hit-rate gain. Its `put` tail cost suggests the next fix should be at the stash-promotion fallback, the remaining cascade source that the gate's fuller target buckets now trigger more often.
