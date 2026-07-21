# Variant v002 Insight

## Helped

- **hit-rate**: The frequency gate fired for re-accessed non-hottest items, giving them early probing residency before they could reach the positional hottest slot — confirming that workload-a contains items that genuinely cycle through target multiple times without ever holding the MRU position, and that providing an alternative promotion path for them reduces their eviction rate.

## Hurt

- **per-op-time**: The extra `lfu_counter` increment and threshold check on every non-hottest target access, plus the occasional `remove_at + promote_to_probing` triggered when the gate fires, raised get p99 (200 ns → 300 ns) and put p50 (100 ns → 200 ns); each frequency-promote event cascades a demotion through the target and stash tiers, adding latency that spreads to co-located keys rather than staying local to the promoted item.

## Takeaway

Adding a frequency gate at threshold ≥ 2 does unlock early probing residency for some re-accessed non-hottest items and produces a real, if modest, hit-rate improvement over v001's positional-only promotion. The latency regression reveals the cost of a low threshold: the gate fires frequently enough to generate a steady stream of cascade evictions, imposing overhead on operations that had nothing to do with the promoted key. The signal here is that the frequency idea has merit, but the threshold is too aggressive for this workload size — raising it, or gating promotion on bucket fullness, would reduce cascade churn and could retain most of the hit-rate gain.
