# Cross-Cycle Insights

## Recurring wins

- **Idiomatic Rust abstractions outperform hand-rolled optimizations.** Across every cycle, the compiler's codegen for standard patterns (e.g., `Option<NonZeroU32>` match arms, ILP-friendly standard hash calls) beat explicit arithmetic rewrites. Micro-optimizations that fight the compiler's assumptions consistently regress latency.
- **Eliminating serial hash dependencies via bit-slicing pays off.** Deriving all sketch row positions from a single hash output by slicing non-overlapping bit windows removes redundant full hash invocations without introducing intra-iteration data dependencies, yielding a measurable win on both hit rate and median latency.
- **Eviction candidate quality is a live signal.** Comparing two LRU-oldest candidates and evicting the lower-frequency one demonstrably improves hit rate, confirming that the baseline eviction selection leaves correctable errors on the table.

## Recurring failures

- **Sketch decay at any tested cadence degrades this workload.** Both an aggressive (`width/2`) and a moderate (`capacity × 10`) reset interval hurt hit rate and latency. The LIRS trace's stable hot working set makes accumulated frequency a reliable admission signal; half-decay blankets hot and cold keys equally, destroying that signal faster than accesses can rebuild it. This axis is exhausted.
- **Sketch structural simplifications backfire.** Flattening to a single allocation introduced cache-set conflicts that more than doubled latency; halving depth degraded collision suppression enough to raise both miss rate and latency. The four-row, multi-allocation baseline is well-balanced for this trace and resists simplification.
- **Node compaction offers no net gain beyond correct encoding.** The `Option<NonZeroU32>` representation is the right encoding (recovering the discriminant-test regression of the raw sentinel approach), but the compacted 24-byte node yields no measurable winner-level improvement over baseline. The node-layout axis is exhausted.

## Open questions

- **Two-phase hashing: does splitting into two seeded half-hashes recover row independence?** The single-hash bit-slice winner showed slight tail regression on put p50 and get p99, likely because four windows of one 64-bit value provide weaker row independence than four independent seeds. Two half-hashes (each covering two rows) may preserve the hash-reduction gain while closing that gap.
- **Can best-of-2 eviction be made latency-neutral?** The hit-rate signal from sampling two eviction candidates is real; the cost is two extra sketch probes on every full-cache `put`. Deferred, amortized, or sampled comparison (e.g., only on a fraction of evictions) may retain most of the benefit without the per-eviction overhead penalty.
- **Are admission/eviction policy parameters (window size, protected/probationary ratio) an untapped axis?** Both the sketch and node-layout axes are now largely exhausted; policy-level tuning has not been attempted and may offer orthogonal gains.

## Per-policy notes

- **TinyLFU sketch (CountMinSketch, four rows):** The baseline four-row depth is load-bearing for collision suppression on this Zipfian trace — depth reduction causes estimation noise that cascades into eviction errors. Multi-allocation row layout avoids cache-set pressure that flat allocation incurs. Hash computation is the one proven optimization lever: bit-slicing a single hash into four windows is a confirmed win, and two-phase hashing is the natural next step.
- **Admission policy:** Frequency-based admission via the sketch is sound for this stable working set. Decay, which was meant to improve freshness, consistently undermines the accumulated frequency signal instead.
- **Eviction policy (LRU within segments):** The LRU-oldest selection is correctable — a best-of-2 frequency comparison improves hit rate — but the overhead of the second probe is currently prohibitive. Cheaper sampling strategies are the next logical step before concluding this axis is fully exhausted.
