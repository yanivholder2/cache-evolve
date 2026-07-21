# Task: cycle-insight-writer

Cycle: 0007
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0007

## Winners

- v003
- v002
- v001

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0007\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0007\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0007\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0007\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0007\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0007\hypotheses\H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle pursued two independent axes — recovering sketch row-independence from v002's single-hash baseline, and refining eviction candidate selection — producing two winners: a two-hash bit-slice approach (v001) that improved hit rate at the cost of p99 tail latency, and an amortised best-of-2 eviction scheme (v002) that recovered both hit rate and per-op-time parity simultaneously. A stricter admission gate (v003) also raised hit rate but incurred a substantial latency regression.

## Patterns

- **Two independent axes both improved hit rate, via different mechanisms.** Better sketch row independence (v001) reduced frequency-estimation collisions, while sampled victim comparison (v002) corrected frequency-blind evictions without touching the sketch. This confirms that hit-rate gains are still available on both the sketch accuracy and the eviction policy sides — they are not redundant.
- **Eviction firing rate is a hidden latency lever.** Whether firing fewer evictions (v002's 7-in-8 skip) or harder-to-satisfy evictions (v003's stricter admission gate), the downstream memory-access pattern shifts enough to move latency measurably. v002 turned this sensitivity into a win by reducing the common path; v003 stumbled by disrupting it without compensating elsewhere.
- **The hash count / row independence trade-off is quantified.** v001 sits between the original four-hash baseline (maximum independence) and the cycle 0003 winner (minimum hashing): the second seeded hash costs p99 tail latency without hurting p50, implying the overhead concentrates on contention and cold-miss paths rather than the warm-hit median. Cheaper independence-preserving transforms remain an open opportunity.
- **Amortised sampling is a reliable pattern.** v002's 1-in-8 probe rate preserved per-op-time while still yielding a real, stable hit-rate improvement — the gain survives sampling, which proves the signal is strong enough to exploit at coarser granularity.

## Open questions

- Is there an amortisation ratio finer than 1-in-8 (e.g., 1-in-4 or 1-in-2) that captures proportionally more of the hit-rate benefit while remaining latency-neutral, or does latency break before 1-in-2?
- Can the second seeded hash in v001 be replaced with a cheaper bijective re-mix of the first hash output (e.g., a single multiply-xorshift step) to recover row independence without the full cost of a second SipHash invocation?
- Would combining v003's stricter admission gate with v002's amortised eviction compound their hit-rate signals — and does the reduced eviction rate from v002 neutralise v003's latency disruption?

## Continuation

Next cycle should push the amortisation ratio of v002's best-of-2 eviction: 1-in-4 is the natural next step, and the key question is whether latency stays neutral at that rate. Separately, v001 motivates a cheaper two-row hash variant — a bijective re-mix of the first hash into a second row pair rather than a full second seeded hash call — which could close the p99 gap while preserving the independence gain. Combining v003's admission gate with v002's amortised eviction is a lower-priority but plausible compound hypothesis once both individual axes stabilize.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
