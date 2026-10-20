# Cycle 0003 Insight

## Cohort outcome

All three variants targeted the two open threads from Cycle 0002 — reducing get-path overhead via early-return guards and improving hit-rate quality via frequency-aware ordering — plus one structural experiment with two-zone partitioning. No variant reached the Winners bar: the position-0 guard traded latency for hit-rate, the two-zone partition regressed on both axes, and the single-swap LFU gained hit-rate without moving per-op-time.

## Patterns

- **Position-0 guard confirms the latency bottleneck but over-constrains promotion.** The early-return for already-hot items halved p99 latency as predicted, validating the write-elimination hypothesis. However, capping promotion to one swap per access prevents mid-bucket items from converging to the front quickly enough on a Zipfian workload, eroding the hit-rate the parent's full bubble had built. The useful insight is directional: the guard itself is sound, and the cap just needs to be wider than one.
- **Single-swap LFU with the aligned struct is a clean hit-rate gain at no latency cost.** Composing the 32-byte struct from Cycle 0002 with a frequency-ordered single swap lifted hit quality by steering eviction toward genuinely cold items without touching per-op-time — confirming that LFU ordering's advantage is eviction-candidate quality, not struct overhead, and that the aligned layout is a neutral-to-positive base.
- **Two-zone partitioning is a definitively closed avenue.** The hot/cold boundary shifted with every insertion, pushing boundary items back into the cold zone and degrading hit rate, while the hot-zone bypass saved no real work because baseline LRU already keeps hot items where removal is O(1). No reformulation of this design is worth pursuing at these bucket sizes.

## Open questions

- Can a bounded bubble loop (cap of 2–3 swaps) recover the hit-rate convergence that the single-swap variants sacrifice, while still short-circuiting the rare long-distance promotions that drive latency on a Zipfian workload?
- Does composing the position-0 guard with a bounded loop (rather than one swap) produce additive wins on both per-op-time and hit-rate, or does even a short loop reintroduce enough overhead to nullify the guard's p99 improvement?

## Continuation

The next cycle should compose three confirmed pieces: the 32-byte aligned struct as the structural base, the position-0 early-return guard for already-hot items, and a bounded promotion loop capped at 2–3 swaps rather than exactly one. This triplet directly addresses the failure mode identified in v001 (single-swap too conservative) while preserving v003's hit-rate quality and v001's tail-latency benefit. Two-zone partitioning should not be revisited. If per-op-time still does not improve after bounding the loop, the next hypothesis should examine whether the measurement floor itself masks the guard's contribution rather than adding further structural complexity.
