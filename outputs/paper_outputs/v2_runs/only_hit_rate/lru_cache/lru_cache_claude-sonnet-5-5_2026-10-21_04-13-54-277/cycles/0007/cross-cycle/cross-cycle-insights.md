## Recurring wins

- Segmented LRU (scan-resistant) was the first real gain and remains the base for the best lines; every later winner builds on it.
- Raising the protected share helped over two cycles (larger beat smaller), then flattened. The curve is now at a plateau.
- Admission filtering (sketch gate) gave small gains; counting rejected misses matters because recurring keys build frequency across retries.
- Simpler designs stay competitive: a doorkeeper-free SLRU plus sketch won against more elaborate variants.
- CLOCK-style rewrites give small hit-rate gains over strict LRU, but they are incremental, not transformative.

## Recurring failures

- Weakening the protected segment (smaller share, direct ghost-to-protected admission) lost hit-rate.
- Probation-side tweaks (demotion placement, probation-only eviction scans, ghost at probation MRU) were inert or matched the parent. Demotions are rare at the high protected share, and some variants were dead by construction.
- Protected-eviction refinements (second-chance bit) and approximate-recency shortcuts (relink skip) gave nothing or lost a little.
- Admission-gate tuning (not counting rejected misses, doorkeeper, longer aging window) is saturated. Slower decay lets stale high-count keys block newer candidates.
- Strict-LRU micro-optimizations are neutral on hit-rate and cannot win while cost is unmeasured.
- Single-constant tweaks are validated on one trace at one cache size, so differences are marginal.

## Open questions

- Metadata-size and per-op-time have been missing from every fitness vector for seven cycles. Is this a harness fault? It blocks judging every cost-motivated variant (compact layout, fast paths, CLOCK, sketch, doorkeeper).
- Do the gains hold on other traces and cache sizes, especially ones with long-distance reuse or less scan pressure? Only one scan-heavy trace has been used.
- Would adaptive segment sizing (ARC-style) beat any fixed share?
- Can frequency that persists across segments, or a CLOCK/sketch hybrid, beat the segmented line in a head-to-head, cost-aware comparison?
- Do the aging window and doorkeeper reset have separate effects? They were changed together.
- Do the compact layout and cheap promotion ideas combine with the segmented winner?

## Per-policy notes

- Strict LRU: the baseline. Hit-rate-preserving rewrites tie it and cannot win.
- Segmented LRU: the best line. The protected share is saturated near 90-95%. Probation is near its minimum useful size and the segment is saturated on this trace.
- Segmented LRU plus ghost list: the direct-to-protected version hurt. The gentler version matched its parent. Drop it unless long-distance reuse is added.
- Sketch admission (TinyLFU-style): marginal gain. Counting rejected misses helps. The doorkeeper is roughly cost-neutral. Keep the aging window near its current length. The doorkeeper-free SLRU plus sketch is the simplicity baseline.
- Frequency-aware victim selection: now has a real signal, but only a small gain. The probation LRU head is already near the best victim.
- CLOCK (one-bit, frequency-aware, plus sketch gate): small hit-rate gains. Size and speed advantages are unverified.
