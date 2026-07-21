# Cycle 0010 Insight

## Cohort outcome

This cycle tested two approaches to reducing dual-chain SLRU bookkeeping overhead (lazy promotion and CLOCK replacement) alongside a further sketch-width doubling. Both structural redesigns won; the sketch-width increase hit a hard L2 cache boundary and was rejected.

## Patterns

- **Both winners targeted the same root cause from different angles.** v001 pruned the costliest SLRU operation (cross-chain promotion on `put`) without changing the two-chain layout; v003 abolished the second chain entirely, replacing it with a single-bit CLOCK reference flag. Both reduced put-path latency, but v003 was the cleaner win: it also shrank the control-plane struct and matched (slightly exceeded) SLRU hit quality, making it a Pareto improvement across all tracked dimensions.
- **CLOCK confirms that SLRU's value is purely second-chance semantics, not structural chain separation.** v003's bounded CLOCK scan — referencing at most 20% of slots per eviction — replicated the protected-segment reprieve without maintaining separate head/tail pointers or a live count. This retroactively explains why protected-zone ratio tuning above 20% yielded no benefit: the two-list bookkeeping was overhead with no return beyond what a reference bit provides.
- **8× sketch width is the L2 ceiling for this capacity range.** v002's 16× doubling pushed the sketch heap past 256 KB, converting a sporadic tail-latency cost into a systematic per-operation slowdown at both p50 and p99. The hit-rate increment at 16× was roughly a quarter of the prior 4×→8× step, confirming collision noise is near-saturated at 8×. This axis is definitively closed.
- **Lazy promotion (v001) is subsumed by CLOCK (v003).** Both hypotheses were valid responses to Cycle 0009's put-path diagnosis, but v003 generalised the fix: eliminating the second chain removes the cross-chain migration entirely rather than just avoiding it on the write path. v001's contribution is rendered redundant by v003's win.

## Open questions

- With the CLOCK eviction scan bound currently fixed at ≈20% of capacity, is there a workload-adaptive or frequency-guided bound that offers better eviction quality under non-Zipfian traces without increasing per-eviction scan cost?
- The window-admission segment (a small recency buffer feeding into the main frequency-filtered pool) is a standard TinyLFU component that has never been explored across ten cycles — could it improve cold-start or scan-heavy access patterns that the sketch alone handles poorly?
- Does v003's CLOCK design interact with the amortised 1-in-8 stricter gate threshold, or do the two mechanisms operate on fully orthogonal paths such that each can be tuned independently?

## Continuation

v003 (CLOCK eviction) is the natural new baseline: it is cleaner than the dual-chain SLRU in every measured dimension and should replace v002 as the parent for future variants. The sketch-width and counter-narrowing axes are both closed. The next cycle should explore genuinely untouched structural territory — the most promising candidate is a window-admission buffer (a small recency-first segment upstream of the main CLOCK pool) to handle cold-start and scan traffic that the frequency sketch currently misclassifies. Alternatively, probing the CLOCK scan bound adaptively (e.g., scaling with recent eviction hit rate) is a low-footprint hypothesis on the existing v003 design.
