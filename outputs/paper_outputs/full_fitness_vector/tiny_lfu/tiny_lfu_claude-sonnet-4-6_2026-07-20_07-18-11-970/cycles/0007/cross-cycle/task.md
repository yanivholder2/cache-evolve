# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted distinct structural axes — sketch aging, node memory layout, and hash computation — but none produced a net winner; each achieved its primary structural goal while incurring a regression that cancelled the benefit.

## Patterns

- **Structural goals met, performance goals missed across the board.** Decay halved counters as intended, node compaction cut metadata size by ~50%, and double-hashing reduced hash invocations — yet all three regressed on at least one fitness dimension, suggesting the baseline is already well-tuned for the cost-sensitivity of this workload and key type.
- **The workload's stable working set invalidated the decay premise.** The LIRS trace's long-lived hot set makes accumulated frequency a reliable admission signal; an aggressive reset interval destroyed that signal faster than accesses could rebuild it, inverting the expected hit-rate gain.
- **Compiler-generated code for idiomatic Rust outperformed explicit arithmetic optimizations.** Both the `Option<usize>`→`u32` sentinel swap and the double-hash serial chain discovered that the compiler's branch-prediction and ILP-friendly codegen for standard abstractions beat hand-rolled alternatives at this scale.
- **Size and speed moved in opposite directions.** Compact node representation improved memory density but doubled hot-path latency, revealing a trade-off that cannot be resolved by layout alone.

## Open questions

- Is there a decay interval (much longer than `width/2`, or entropy-triggered) that preserves the frequency signal on stable working sets while still aging stale counts?
- Can node compaction recover its speed loss if `NonZero<u32>` restores niche optimization and eliminates explicit sentinel comparisons?
- Would a single-pass multiply-shift or seeded hash avoid the serial `h1→h2` data dependency while still reducing total hash work per sketch call?

## Continuation

Next cycle should deprioritize the `width/2` decay cadence — it is clearly too aggressive for this trace — and instead either experiment with much longer or adaptive intervals, or focus on a different axis entirely. The u32 node compaction trade-off is worth one more attempt with `NonZero<u32>` before abandoning; if that also regresses latency, drop the layout approach. For hash reduction, any new hypothesis must eliminate serial inter-hash dependencies, as ILP loss dominates the gain from fewer hash calls on cheap integer keys.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants targeted the CountMinSketch or node memory layout — one refining the node-encoding fix from the prior cycle, one consolidating sketch allocation, and one trimming sketch depth — but none crossed the winner threshold; the closest (NonZeroU32 node encoding) merely returned to baseline parity without surpassing it.

## Patterns

- **The CountMinSketch resists structural simplification on this workload.** Both sketch modifications failed in opposite ways: flattening to a single allocation introduced cache-set conflicts that more than doubled latency, while halving depth degraded collision suppression enough to hurt hit rate and latency simultaneously. The baseline's four-row, multi-allocation design is well-balanced for the trace's capacity and access pattern.
- **v001 confirmed NonZeroU32 as the correct node encoding, but node compaction is now fitness-neutral.** Recovering v002's latency regression with idiomatic `Option<NonZeroU32>` match arms proved the discriminant-test codegen was the culprit — not node footprint itself. However, the 24-byte node yields no measurable winner-level improvement over baseline, exhausting the node-layout axis.
- **Flat allocation is not a reliable substitute for multiple independent heap segments.** The pointer-indirection cost of `Vec<Vec<u32>>` was lower than predicted; the real cost at runtime is the cache-set pressure created by the large row strides in the flat layout, which the separate-allocation baseline avoided by chance of heap placement.
- **Depth redundancy in the sketch is load-bearing, not padding.** The four rows provide meaningful collision suppression on this Zipfian trace — removing two rows at this sketch width causes enough estimation noise to cascade into eviction errors and overall latency increase.

## Open questions

- Is there a sketch modification that preserves four-row accuracy while reducing the per-call hashing cost — e.g., computing all four positions from a single seeded hash rather than four independent hash invocations?
- Are there fitness gains still available on the admission or eviction logic side (e.g., window/protected segment sizing, admission threshold tuning) that are orthogonal to the sketch and node axes now both largely exhausted?

## Continuation

Next cycle should abandon sketch-structure changes (both flat allocation and depth reduction are definitively closed) and node-layout changes (NonZeroU32 is the correct encoding but offers no further gains). The one remaining open thread from Cycle 0001 — adaptive or longer-interval sketch decay — has not been tested and should be prioritized: the stable working set may tolerate a much longer reset window that ages stale keys without disturbing the hot-set frequency signal. Alternatively, shifting focus to admission and eviction policy parameters (window size, protected/probationary ratio) offers an unexplored axis.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle tested three independent axes: sketch decay cadence, hash-work reduction, and eviction candidate selection. Only the single-hash bit-slicing approach (v002) earned a winner, by eliminating three redundant hash computations per sketch call; the decay variant repeated a prior failure pattern, and the best-of-2 eviction variant improved hit rate but not enough to offset its latency cost.

## Patterns

- **Sketch decay is definitively closed.** The moderate `capacity × 10` interval — intentionally longer than the cycle 0001 failure — still hurt both hit rate and latency: blanket half-decay penalises hot keys and cold keys identically, and the periodic full-table sweep is large enough to pressure L2/L3, raising the median. Two intervals have now failed on this workload; the stable LIRS working set makes accumulated frequency a reliable signal that decay degrades rather than improves.
- **Hash-work reduction wins when it eliminates serial dependencies.** Bit-slicing a single SipHash output into four non-overlapping windows removes three redundant full hash invocations with no intra-iteration data dependency, confirming the cycle 0001 prediction about ILP. The put p50 and get p99 both showed slight regression, suggesting that four non-overlapping 16-bit windows of one 64-bit value provide marginally weaker row independence than four independently seeded hashes — visible in collision-sensitive tail paths but not in the overall median.
- **Eviction logic is a live axis with a real signal.** v003 is the first variant in any cycle to improve hit rate through candidate selection rather than sketch changes. Comparing the two LRU-oldest entries and evicting the lower-frequency one corrects a real class of admission error, but firing two extra sketch probes on every full-cache `put` makes the latency trade-off unfavorable at current eviction frequency.

## Open questions

- Can two-phase hashing (e.g., two seeded hashes, each sliced into two row positions) recover the independence advantage of four independent hashes while still halving the hash cost — preserving v002's gain while closing its tail regression?
- Is there an amortized or deferred form of best-of-2 eviction that batches or lazily skips the second sketch probe to reduce the per-eviction overhead while retaining most of the hit-rate benefit?

## Continuation

Next cycle should build on v002's win: the single-hash bit-slice approach is the new baseline to beat, and the open thread is whether replacing it with two half-hashes (each covering two rows with an independent seed) recovers row independence without re-introducing full redundant hashing. The eviction-axis hypothesis from v003 should be carried forward in a cheaper form — deferred or sampled frequency comparison rather than a mandatory probe on every eviction — since the hit-rate signal it found is real. Decay should not be revisited; both tried intervals have failed and the axis is exhausted.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
