# Cross-Cycle Insights

## Recurring wins

- **Hit-rate improvements are achievable** — both LFU-semantics and wider-bucket geometry produced measurable hit-rate gains on the LIRS trace, confirming these axes hold real signal.
- **Struct trimming lands its primary target** — removing unused metadata fields reliably reduces struct size as predicted; the structural win is repeatable.

## Recurring failures

- **Linear-scan eviction kills latency** — any policy that scans entries to find min/max (e.g., frequency counters) or widens the scan surface (more entries per bucket) consistently raises per-op-time, blocking the win regardless of other gains.
- **Struct-size reduction does not transfer to speed** — smaller structs alter cache-line packing in ways that can raise get latency even when the removed field is never read; metadata-size and runtime-performance objectives conflict at this occupancy level.
- **Per-op-time is the universal disqualifier** — in every variant so far, latency regression has been the sole reason a candidate failed to win, even when its primary target metric improved.

## Open questions

- Can a constant-time LFU structure (frequency-bucketed doubly-linked list) recover the hit-rate win without the O(n) scan penalty?
- Do LFU semantics and wider-bucket geometry compose additively for hit rate, or do they interfere?
- Is the cache-line regression from struct shrinkage fixable with explicit alignment, or is it inherent at current bucket occupancy?

## Per-policy notes

- **LFU / frequency-based eviction** — produces scan-burst resilience and real hit-rate lift on LIRS traces, but current O(n) implementation is latency-incompatible; a constant-time variant is the logical next step.
- **LRU / default eviction** — serves as the baseline; no dedicated results yet, but struct-trim experiments run on top of it reveal cache-line sensitivity.
- **Wider-bucket geometry (32×64)** — hit-rate gains are corroborated; latency cost appears tied to the larger per-lookup scan surface rather than geometry per se; worth revisiting once eviction cost is controlled.
