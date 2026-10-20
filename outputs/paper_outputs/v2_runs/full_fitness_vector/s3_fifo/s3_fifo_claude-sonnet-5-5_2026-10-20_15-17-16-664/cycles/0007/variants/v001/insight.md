# Variant v001 Insight

## Helped

- per-op-time: `put` improved sharply, at both the typical and tail latency. A miss now probes the small/main index once, and the separate ghost table only after that. Ghost inserts and trims no longer churn the main index, so rehash and resize pressure drops.
- per-op-time: `get` improved slightly at both the median and the tail. It now probes only the small/main index, which holds no ghost entries. Probe chains are shorter, but the effect is modest.
- hit-rate: Unchanged, as the hypothesis required. Eviction order, promotion threshold, small ratio and ghost bound were all preserved. The sequence-number check on ghost removal keeps stale ring slots from evicting live ghost entries.

## Hurt

- metadata-size: The whole-cache struct grew a little, because the extra ghost map adds its own table header and hasher state. The per-entry summary did not change. The claimed footprint reduction did not show up in this metric. It only measures the struct's inline size, not heap allocation, so the effect of removing ghost keys from the main table is invisible here.

## Takeaway

Separating ghost membership from the tagged index paid off mainly on `put`, where ghost bookkeeping was the cost, and only marginally on `get`. The footprint claim was not confirmed, because the metric counts struct bytes, not heap. Hit rate stayed identical, so splitting a side structure out of a hot index is a safe, behavior-preserving way to cut write-path cost.
