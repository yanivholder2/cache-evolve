# Cross-Cycle Insights

## Recurring wins

- **ILP-friendly hash work reduction pays off when serial dependencies are eliminated.** Bit-slicing one SipHash output (c0003 v002) and replacing a second SipHash with a bijective multiply-xorshift remix (c0005 v002) both won by removing intra-iteration data dependencies. Each step that preserved instruction-level parallelism on the hash path delivered measurable put-path gains.
- **Amortised sampling is a reliable pattern for expensive per-eviction work.** Firing auxiliary logic (victim frequency comparison, admission gate) on 1-in-8 evictions instead of every eviction has produced winners in both c0004 (v002, best-of-2 eviction) and c0006 (v002, stricter admission gate). The signal is strong enough to survive coarse sampling.
- **Admission gate tightening wins only when residency churn stays low.** The strict `incoming_freq > victim_freq + 1` gate succeeds at hit-rate improvement when amortised to 1-in-8 and does not issue extra sketch probes; it fails when applied every eviction or combined with additional probe calls.
- **Idiomatic Rust abstractions outperform hand-rolled arithmetic.** `Option<NonZeroU32>` match arms (c0002 v001) and the compiler's standard codegen consistently beat manual sentinel comparisons and explicit arithmetic optimisations on this workload.

## Recurring failures

- **Sketch decay at any tested interval degrades this workload.** Both an aggressive `width/2` cadence (c0001) and a moderate `capacity × 10` cadence (c0003) hurt hit rate and latency. The LIRS trace's stable hot set makes accumulated frequency a reliable admission signal; periodic half-decay destroys that signal faster than accesses rebuild it. This axis is fully closed.
- **Flat single-allocation CountMinSketch layout causes cache-set conflicts.** Collapsing multi-row vectors into one contiguous allocation (c0002 v002) more than doubled latency. The separate-allocation baseline avoids row-stride cache-set pressure by chance of heap placement; flat layouts cannot replicate that at this sketch width.
- **Halving sketch depth hurts both accuracy and latency.** Reducing from four rows to two (c0002 v003) caused enough collision noise to cascade into eviction errors and raised latency. Four rows is load-bearing for Zipfian collision suppression at this capacity.
- **Bijective remix chains cannot substitute for independent hash seeds.** A multi-step remix rooted at a single seed (c0006 v003) produces correlated row positions that raise sketch collision noise and stalls out-of-order execution more than a second independent SipHash would. Row independence from a shared root is insufficient.
- **Adding extra sketch probes on the eviction path erases amortisation savings.** c0006 v001 combined a 1-in-8 fire rate with two extra `estimate` calls on that path; latency doubled across both get and put, even on code paths that never execute the new logic. The struct-size growth likely shifted a cache-line boundary, but execution overhead on the infrequent path amplified the penalty decisively.
- **Probe amortisation rate tighter than 1-in-8 breaks latency.** 1-in-4 (c0005 v001) visibly broke p50 and p99. The axis is bracketed: 1-in-8 is the operating point; finer rates are not viable unless estimates are reused rather than freshly issued.

## Open questions

- **Can best-of-2 victim selection be achieved without growing the struct?** If the eviction counter can be packed into existing alignment padding (rather than adding 8 bytes and shifting a cache-line boundary), the layout penalty that wrecked c0006 v001 may disappear and the hit-rate signal from c0004 v002 may become accessible at the current baseline.
- **Are structural policy parameters (window ratio, protected/probationary balance, promotion threshold) a live axis?** No cycle has touched these levers. With sketch, hash-work, node layout, eviction-victim selection, and admission-gate axes substantially explored, this may be the remaining untapped frontier.
- **Is there a softer admission gate that limits residency churn while still capturing hit-rate benefit?** Applying the stricter comparison only on the amortised-probe path (without extra probes) or using a fractional rather than integer threshold has not been tested as a standalone variant.
- **Can two seeded hashes covering two rows each (rather than four independent hashes) recover full row independence at half the hash-invocation cost?** c0004 v001 showed this sits between the extremes in accuracy; the p99 tail cost remains the open question.

## Per-policy notes

- **TinyLFU / CountMinSketch (admission):** Sketch structure is well-optimised at four rows, separate allocations, and a bijective-remix single-hash baseline. Decay is closed. Hash-independence gains from a second seeded hash are real but come with p99 tail cost concentrated on contention paths. The four non-overlapping 16-bit windows from one 64-bit value provide slightly weaker row independence than four independent hashes, visible only in collision-sensitive tail paths.
- **Eviction policy (victim selection):** Best-of-2 at 1-in-8 amortisation is a proven hit-rate lever (c0004 v002 winner). It requires no extra sketch probes to work; adding probes on the same path breaks it. Struct-size growth from an eviction counter is a latency risk if it crosses a cache-line boundary — packing into padding is the prerequisite for revisiting this approach.
- **Admission gate:** Amortised strict-gate (c0006 v002 winner) is the current baseline. It improves hit rate with no latency regression by avoiding extra probes and limiting residency churn to 1-in-8 evictions. Any further tightening must stay below the churn threshold.
- **Node layout:** `Option<NonZeroU32>` is the correct encoding (c0002 v001). The 24-byte node is fitness-neutral vs. baseline; the axis is exhausted unless a future layout change enables a larger structural improvement elsewhere.
