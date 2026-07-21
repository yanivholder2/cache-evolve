# Task: cycle-insight-writer

Cycle: 0003
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0003

## Winners

- v002

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0003\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0003\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0003\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0003\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0003\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0003\hypotheses\H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
