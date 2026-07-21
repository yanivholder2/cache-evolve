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
