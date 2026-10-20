# Task: cycle-insight-writer

Cycle: 0008
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0008

## Winners

- v001
- v002

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0008\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0008\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0008\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0008\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0008\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0008\hypotheses\H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle pushed three independent threads from Cycle 0004: tightening the best-of-2 eviction probe rate, replacing the second SipHash with a cheap bijective remix, and stacking a stricter admission gate atop the remix base. Only the bijective-remix variant won, by eliminating the put-path serial hash dependency while leaving hit rate essentially unchanged.

## Patterns

- **The probe-amortisation axis is now bracketed and closed.** 1-in-8 is the upper bound on probe frequency that preserves latency neutrality; 1-in-4 broke p50 and p99 visibly. No further hit-rate gain is available along this axis without reusing already-computed estimates rather than issuing fresh sketch probes.
- **The count-min sketch needs less hash independence than theory suggests.** A three-instruction multiply-xorshift remix produces row mixing that is sufficient for accurate admission decisions on this workload, making the second full SipHash invocation wasteful. The bijective remix is a strict improvement: it removes a serial data dependency on the put path with negligible accuracy cost.
- **Admission gate latency is a residency problem, not an execution-overhead problem.** The stricter gate doubled p50 for both get and put by altering which nodes occupy cache lines — an effect the 1-in-8 eviction amortisation is structurally unable to cancel. The two mechanisms target independent failure modes, so v002's amortisation cannot neutralise v003's residency disruption.
- **Additive hit-rate signals do not imply additive latency budgets.** v003 confirmed that combining two independently beneficial mechanisms can compound their latency costs even when their hit-rate gains compound cleanly.

## Open questions

- Is there a softer admission threshold — applied only on the amortised probe path rather than every eviction — that reduces the number of nodes whose residency changes while still capturing some of the hit-rate benefit?
- With the sketch, hash-work, and eviction-amortisation axes all largely exhausted, are there meaningful gains remaining on the window/protected-segment sizing or probationary-to-protected promotion threshold?

## Continuation

v002's bijective-remix baseline is the new starting point. The amortisation-rate axis (1-in-8 vs. 1-in-4) is definitively closed — pursue it further only if a way to reuse pre-computed frequency estimates appears. The admission gate is still a live thread but requires a softer mechanism: applying the stricter comparison only on the amortised-probe path, or using a fractional threshold, to limit residency churn. If both of those threads quickly exhaust, the next fruitful direction is structural policy parameters (window ratio, protected/probationary balance) that have not been touched in any prior cycle.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle pursued three threads from prior cycles: combining best-of-2 victim selection with the bijective-remix base, amortising the strict admission gate to 1-in-8 evictions, and extending the remix-chain approach to the two-seeded-hash incumbent. Only the amortised strict-gate variant (v002) won, improving hit rate while holding latency at the parent's level.

## Patterns

- **Amortised gating works when it avoids extra sketch probes.** v002 applied a stricter admission threshold on every 8th eviction without issuing any new sketch probes, and it won cleanly on both hit rate and latency. v001 combined the same struct counter with two additional `estimate` calls on the 1-in-8 path — that combination doubled p50 across get and put, even though the extra work touches only one in eight evictions. The divergence shows that adding extra sketch probes matters more than the amortisation rate suggests, especially when compounded with a struct-layout shift.
- **A shared struct-size increase can have asymmetric latency impact.** Both v001 and v002 added the same 8-byte eviction counter, growing `TinyLfuCache` from 152 to 160 bytes. v002 still won; v001's latency doubled across both get and put even on code paths that never execute the new logic. This suggests the cache-line boundary shift is real, but execution overhead on the 1-in-8 path amplifies it enough to turn a manageable penalty into a decisive regression.
- **Remix chains cannot substitute for independent hash seeds.** v003 attempted to replace the second seeded SipHash with a three-step bijective chain rooted at a single seed. Both accuracy and throughput regressed: a common hash root produces correlated row positions in the count-min sketch, raising collision noise, while the sequential multiply–rotate–XOR chain stalls out-of-order execution more than a second independent SipHash invocation does. The two-seeded approach's value is genuine seed independence, not just hash-count reduction.
- **Residency churn rate, not threshold strictness, is the admission-gate latency lever.** Cycle 0005 showed that firing `incoming_freq > victim_freq + 1` on every eviction doubled p99 through residency churn. v002 confirmed this by recovering near-parent latency at 1-in-8 fire rate. The implication is that any further admission-gate tightening must stay below the churn threshold rather than soften the comparison itself.

## Open questions

- Can best-of-2 victim selection (the goal of v001) be achieved without growing the struct — e.g., by reusing an existing padding gap for the counter — to avoid the cache-line boundary penalty that amplified its probe overhead?
- With sketch accuracy, hash-work, eviction-victim selection, and admission-gate axes all substantially explored, are structural policy parameters (window ratio, protected/probationary balance, promotion threshold) the remaining untapped frontier?

## Continuation

v002's amortised strict-gate is the new baseline. The next cycle should probe whether best-of-2 victim selection can be layered on top without a struct-size increase — if the counter can be packed into existing alignment padding, the cache-line penalty that broke v001 may disappear. The remix-chain-as-hash-replacement thread is definitively closed: do not revisit it. If the struct-layout avenue is exhausted quickly, pivot to structural policy parameters (window/segment ratio, promotion threshold) as the next independent axis — none of the prior six cycles have touched these levers.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
