# Variant v001 Insight

## Hurt

- **hit-rate**: Replacing occupancy-responsive sizing with ghost frequency decay fails to recover the hit-rate lost by removing adaptive small-queue growth. Adaptive sizing actively protected items during load phases; frequency decay only recovers on re-access, which cannot replace that active protection.
- **per-op-time (get)**: Ghost queue now stores frequency state and requires per-hit position lookup, removal, and arithmetic boosting (ghost_freq + 3), adding ~43ns of overhead compared to the simple key-based ghost search in v002.

## Helped

- **per-op-time (put)**: Removing occupancy-ratio checks eliminates a frequent recomputation, yielding a marginal ~4ns improvement, though too small to offset the hit-rate and get-latency losses.

## Takeaway

Ghost frequency decay with lenient retention (-3 depth) cannot substitute for adaptive queue sizing as a hit-rate recovery mechanism. While frequency decay is theoretically sound for preserving warm candidates, v002's occupancy-aware growth was *actively* adjusting admission barriers to capture more items during low-load phases—a dynamic behavior that frequency decay's passive re-access recovery cannot replicate. Additionally, the implementation's per-operation overhead in the get path (searching and boosting ghost entries) contradicts the "lean, overhead-free" design goal, combining the latency cost of complexity with the hit-rate failure of insufficient adaptation.
