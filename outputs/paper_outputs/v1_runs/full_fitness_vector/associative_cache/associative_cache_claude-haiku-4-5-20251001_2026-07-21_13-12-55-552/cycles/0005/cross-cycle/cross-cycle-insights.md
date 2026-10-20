# Cross-Cycle Insights

## Recurring wins

- **Simple policies with tight implementations outperform complex policies with per-hit overhead** — ClassicLRU, FIFO, and TimestampLRU all maintained Zipfian hit-rate (0.6611–0.664) while policy variants adding per-hit bookkeeping (Swap, segment-based LRU, FIFO with timestamp victim selection) regressed 2×–4× on latency. The pattern holds uniformly across cycles 0001–0005.

- **Micro-optimization focused on the hit path can recover wide-bucket geometry benefits** — Cycle 0004's v003 (early-exit + cache-line alignment on 32×64) achieved 50% latency improvement on get operations and 33–50% on puts while preserving hit-rate gain (0.664). First variant to decouple geometry scaling from latency penalty.

- **Zipfian workload is indifferent to eviction policy semantics** — Hit-rate remained stable across ClassicLRU, FIFO, TimestampLRU, and LFU (0.6611–0.664 range), confirming that workload access skew alone drives hit-rate; policy details matter less than implementation cost.

## Recurring failures

- **Asymptotic complexity wins evaporate under cache-level micro-architecture** — Swap's O(1) removal (cycle 0001), TimestampLRU's O(1) timestamp updates (cycle 0002), and segment-based LRU's bounded eviction (cycle 0004) all regressed on latency despite theoretical advantages. Constant factors—cache misses, branch misprediction, memory latency—dominate at benchmark scale.

- **Policy changes optimizing eviction at the expense of hit-path efficiency fail** — Cycle 0004's FIFO with timestamp victim selection (O(n) scan per eviction) and segment-based LRU (per-op metadata) both suffered 2×–4× latency regressions. Eviction is rare; hits are common—optimizing the wrong path is a losing trade-off.

- **Micro-optimizations do not generalize across bucket geometries** — Cycle 0005 showed that early-exit and cache-line alignment worked on 32×64 but hurt on 128×16 (get p50 +45%, p99 doubled) and inverted on 16×128 (put 3× slower, get p99 doubled). Optimization is finely tuned to specific geometric profile, not reusable.

- **Wider buckets hit an accelerating latency wall** — Cycle 0002's 64×32 cost 45% latency for 0.24% hit-rate gain; cycle 0003's 32×64 spiked p99 sharply. Intra-bucket linear search becomes the bottleneck as bucket size grows; the trade-off is not tractable via geometry alone.

## Open questions

- **Why does ClassicLRU's O(n) vector manipulation beat O(1) alternatives?** — At small bucket sizes (16 items), cache coherency, branch prediction, or compiler loop optimization may make asymptotic analysis irrelevant. Root cause requires profiling.

- **Is 32×64 with micro-ops truly a Pareto win across workloads, or a narrow local optimum?** — Cycle 0004 claimed leadership but cycle 0005's failure to generalize micro-optimizations raises doubt. Needs empirical validation across non-Zipfian distributions.

- **Have we exhausted the geometry-latency trade-off space?** — Geometry tuning and micro-optimization show diminishing returns. Orthogonal approaches—cache-oblivious designs, learned hashing, adaptive policies, cuckoo hashing—might break the current frontier.

- **How robust is any variant across different workloads, cache sizes, or multi-core contention?** — All testing is on a single Zipfian benchmark; generalizability unknown.

- **Why does FIFO maintain high hit-rate without recency tracking?** — Is Zipfian skew sufficient to sustain hit-rate via FIFO alone, or does this generalize?

## Per-policy notes

- **ClassicLRU (baseline 128×16)**: Stable hit-rate (0.6611), finely optimized implementation. All single-policy variants regressed, suggesting workload fits recency naturally or implementation tuning is already saturated. Remains fitness baseline through cycle 0005.

- **LFU**: Frequency tracking added complexity without hit-rate improvement (cycle 0001). Zipfian workload's recency structure makes frequency semantics redundant.

- **FIFO**: Maintained ClassicLRU hit-rate (0.6611) without per-hit bookkeeping on basic geometry (cycles 0002–0003), confirming Zipfian workload doesn't require recency semantics. However, cycle 0004's FIFO variant with timestamp victim selection regressed sharply—efficient victim selection still carries per-operation cost; "zero bookkeeping" claim was misleading.

- **TimestampLRU**: Hit-rate matched ClassicLRU, but timestamp updates added ~40% latency (cycle 0002). Orthogonal bookkeeping method; not a latency solution.

- **Segment-based LRU**: Attempted to bound eviction scan depth via per-operation segment metadata, yielding ~4× latency regression (cycle 0004). Eviction is rare; optimizing it at hit-path cost is backward.

- **Geometry variants**: Fine-grained buckets (256×8) regressed both hit-rate and latency. Wide buckets (64×32, 32×64, 16×128) exposed accelerating latency penalties outweighing hit-rate gains, except 32×64 with micro-optimization. Geometry alone insufficient; implementation maturity dominates.
