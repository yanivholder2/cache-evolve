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
