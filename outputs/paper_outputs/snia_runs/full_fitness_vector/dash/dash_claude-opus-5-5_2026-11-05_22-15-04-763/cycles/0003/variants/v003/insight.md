# Variant v003 Insight

## Helped

- hit-rate: Normal-tier victims now go into stash at the MRU end (`InsertPosition::Back`) instead of index 0, where they would be the next stash victim. Each demoted item gets a full stash residency, and one hit in that window is enough to send it back to target. The gain shows up in all four trace/size cells, and it is largest at the 131072 size and on `hm_0`. So the retention effect works without the two-hit gate.

## Hurt

- per-op-time: `get` p50 did not stay near the baseline. It rose to the band the hypothesis gives for the gated variants, even though this variant has no gate state and no two-touch path. A likely cause is that more lookups now find their key in stash, and each stash hit takes the costly `remove_at` + `promote_to_target` + `demote_from_normal_bucket` route. That cost scales with how well stash retains items, not with the gate. `put` p50 and p99 also rose, though the put path was not edited. Part of the latency shift may therefore come from stash being more crowded, or from noise between runs, rather than from `get` alone.

## Takeaway

Moving demotion to stash MRU is a real hit-rate lever on its own, so the earlier hit-rate gain did not depend on the gate. The hypothesis's latency claim is falsified: removing the gate did not bring back baseline `get` latency. The cost most likely comes from the higher stash-hit traffic that MRU demotion creates. If that holds, cheaper `get` has to come from a cheaper stash-hit promotion path, not from removing the gate. Metadata size did not change because no per-item state was added.
