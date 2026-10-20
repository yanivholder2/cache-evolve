# Cycle 0008 Insight

## Cohort outcome

This cycle explored three independent threads: the compound u16+8× sketch (Cycle 0007's top priority), a protected LRU segment to shield proven-hot residents from eviction, and double-incrementing the sketch on confirmed hits to steepen the frequency gradient. The protected-segment variant and the u16+8× compound both won; the double-increment variant lost on both hit rate and latency.

## Patterns

- **Policy structure is a new, orthogonal hit-rate lever.** The protected-segment variant (v002) delivered a modest but real hit-rate gain and a get-path latency improvement by re-positioning hot-chain hits rather than relinking them through the full probationary chain. This is the first cycle to touch structural policy parameters, and it validated that the LRU segment split adds value independently of all prior sketch and gate tuning.
- **Counter narrowing is a diminishing-returns axis.** The u16+8× compound (v001) achieved its memory target — sketch heap halved to match the u32 baseline — yet hit rate was unchanged rather than recovering v003's +0.36 pp gain, and put p99 still regressed. The zero-extension cast cost on the eviction path cancels the L2 footprint savings at this capacity scale.
- **Proportional frequency amplification does not shift admission-gate orderings.** Doubling the per-hit sketch increment (v003) fails because cold residents accumulate the boosted signal at the same relative rate as incomers accumulate misses; the `incoming_freq > victim_freq` comparison is structurally unaffected, and the overhead falls on the majority of operations (≈74% hits), making it a pure latency cost.
- **Struct size is a compounding risk for the protected-segment approach.** v002 grew `TinyLfuCache` by 47% (152 → 224 bytes), driven by the `prot_flags` Vec and four new fields. The hit-rate benefit is real but the structural cost will compound with higher capacities or write rates.

## Open questions

- Can the protected-segment flag storage be made more compact — e.g., a bitset or reuse of existing node metadata — to reduce the struct-size growth without abandoning the segment split?
- What is the optimal protected-segment ratio? 20% was chosen but not compared against 10%, 30%, or an adaptive size; the hit-rate/struct-overhead trade-off may peak at a different fraction.
- Does combining the protected segment (v002) with the amortised admission gate compound their independent hit-rate benefits, or does the gate become redundant once hot items are shielded from the victim pool?

## Continuation

The protected-segment axis (v002) is the highest-priority live thread: explore more compact flag representations to reduce struct bloat, and probe segment-size ratios beyond 20%. The u16 counter axis is effectively closed — the heap target is achievable but the latency benefit does not materialise at these sketch sizes; do not revisit unless a way to eliminate the per-access cast overhead emerges. The double-increment thread is definitively closed. A natural compound for the next cycle is v002's protected segment layered onto the amortised-gate baseline, with attention to whether the gate adds marginal value once residents are shielded.
