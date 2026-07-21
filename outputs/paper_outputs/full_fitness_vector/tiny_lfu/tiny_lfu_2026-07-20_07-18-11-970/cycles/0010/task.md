# Task: cycle-insight-writer

Cycle: 0010
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0010

## Winners

- v003
- v001

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0010\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0010\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0010\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0010\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0010\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0010\hypotheses\H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle explored three independent axes — best-of-2 victim selection on the amortised gate path, shrinking sketch counters from u32 to u16, and doubling the sketch width factor. All three variants were accepted as winners despite mixed results: v001 was latency-neutral but gained nothing on hit rate; v002 halved the sketch heap but paid a p99 latency increase; v003 raised hit rate but doubled p99 tail latency.

## Patterns

- **Best-of-2 victim selection at the LRU cold end is a closed axis.** v001 confirmed that both the LRU head and its immediate neighbour are equally cold relative to the working set — the admission gate comparison against the incoming key is the operative filter, not which cold candidate is chosen. The hypothesis is refuted even under the optimal implementation that eliminates the redundant estimate call.
- **Sketch width is a genuine hit-rate lever; counter width is not a free-latency lever.** v003 showed that halving expected bucket collision rate (4× → 8× width) does lift hit rate, confirming that collision noise was suppressing admission of genuinely hot keys. v002 showed that halving the heap via u16 counters does not automatically improve latency — the per-access zero-extension overhead outweighs the cache-footprint reduction at these sketch sizes.
- **A compound u16 + 8× width sketch is an untested configuration at the same heap size as today's baseline.** The current 4× u32 sketch occupies 64 KB. An 8× u16 sketch would also occupy 64 KB while halving collision rate. Neither this cycle nor any prior cycle has evaluated this combination, and it avoids the competing penalties that broke v002 and v003 individually.
- **p99 tail latency is the binding constraint, not p50.** Both v002 and v003 left p50 largely unaffected while moving p99 significantly — reinforcing that the sketch is on a sporadic, high-cost code path rather than the hot median path.

## Open questions

- Does an 8× u16 sketch (same 64 KB heap as the u32 4× baseline) achieve v003's hit-rate gain while avoiding v002's per-access widening cost, because a narrower element type at doubled width keeps the same total bytes in cache?
- Are structural policy parameters — window/protected segment ratio, probationary-to-protected promotion threshold — genuinely unexplored levers, or do they interact with sketch accuracy in ways that require the sketch to be tuned first?

## Continuation

The natural next hypothesis is the u16 + 8× width compound: the combined sketch heap matches the pre-existing 64 KB baseline, so the cache-footprint penalty of 8× width is neutralised while the lower collision rate of v003 and the narrower counter access of v002 may each contribute independently. This is the highest-priority untested thread. The best-of-2 victim selection axis is definitively closed and should not be revisited. If the u16 + 8× compound exhausts quickly, shift to structural policy parameters (window ratio, segment balance, promotion threshold) — none of the prior seven cycles have touched these levers.


### Cycle 0008

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle focused on refining the protected-segment design from Cycle 0008: compacting the flag storage to a packed bitset, layering the segment onto the 8× sketch + amortised-gate parent, and probing a wider 30% protected fraction. The bitset compaction and the compound protected + 8× sketch variant were accepted as winners; the wider protected zone regressed on both dimensions.

## Patterns

- **The fitness model is blind to off-struct heap savings.** v001's bitset compaction reduced the `prot_flags` backing allocation 8× with no change to hit rate or per-op time, yet won by not regressing. The struct-level metrics used for evaluation cannot distinguish this from a no-op. Any further heap-footprint changes will be invisible until a separate allocation-size metric is wired into the fitness function.
- **The 20% protected fraction is near-optimal for a Zipfian trace.** v003 confirmed that raising the zone to 30% neither shields more hot keys nor reduces eviction pressure — it simply fills extra protected slots with lukewarm items while shrinking the probationary pool enough to increase churn and hurt both hit rate and latency. The optimal ratio lies at or below 20%.
- **The admission gate and the protected segment are partially redundant.** v002 combined the segment split with the amortised gate and 8× sketch, but the per-insight observation noted that the gate already shields hot keys effectively enough that the structural refuge adds only a marginal hit-rate increment while meaningfully taxing the put path through dual-list bookkeeping. The compound won, but the incremental benefit of the segment on top of a well-tuned gate is small.
- **Put-path latency is the binding cost of SLRU bookkeeping.** Both this cycle's segment variants paid elevated put p50/p99. Every full-cache put must now conditionally walk and relink two chains; this is an inherent cost of the dual-list design that is independent of the protected-zone ratio.

## Open questions

- Can the dual-list put-path overhead be reduced — e.g., by deferring promotion to `get` only and using a simpler flag check at eviction time — without sacrificing the hit-rate benefit of the segment split?
- Does combining the compact bitset (v001's representational fix) with the compound 8× sketch + protected segment (v002) produce any measurable latency improvement now that the flag-array heap is 8× smaller and more cache-friendly on the eviction path?
- With the protected-segment ratio confirmed at ≤20% and the put-path cost measured, is there a different structural lever — e.g., a clock-hand or generational eviction policy — that delivers segment-like admission filtering with lower per-operation bookkeeping?

## Continuation

The protected-segment axis is approaching its local optimum on this workload: the 20% ratio is near-correct, the bitset compaction is implemented, and the gate + segment compound has been tested. The next cycle should assess whether the put-path dual-list cost can be reduced by restricting segment promotion to the `get` path only (lazy promotion), which would eliminate most of the dual-list traversal from write operations. If that path yields no latency improvement, the protected-segment thread should be treated as complete and attention should shift to genuinely unexplored structural levers — generational or clock-based eviction — that may deliver similar hit-rate gains with lower per-operation overhead.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
