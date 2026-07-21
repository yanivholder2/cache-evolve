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
