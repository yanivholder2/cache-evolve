# Cycle 0004 Insight

## Cohort outcome

This cohort pursued two competing strategies for managing wide-bucket latency: policy changes to reduce per-hit overhead (FIFO with timestamp victim selection, segment-based LRU), and implementation-level micro-optimizations (early-exit scanning and cache-line alignment) applied to ClassicLRU. The policy variants failed with latency regressions of 2×–4×. Only v003—which abandoned policy change and instead optimized the implementation—achieved a net improvement, preserving the 32×64 geometry's hit-rate gain (0.664) while halving tail latencies through early-exit and cache-aligned layouts.

## Patterns

- **Policy changes incur hidden per-operation costs that outweigh asymptotic gains**: Both v001 (FIFO) and v002 (segment-based LRU) suffered from changes that were theoretically sound but practically expensive. v001's timestamp-based victim selection replaced ClassicLRU's O(1) position lookup with O(n) bucket scan, adding 100–200 ns per operation. v002 added per-operation segment metadata maintenance that yielded nearly 4× latency regression despite bounding eviction scans. Both confirm the pattern from prior cycles: optimizing the eviction path must not burden the critical hit path.

- **Micro-optimization succeeds where policy fails**: v003 kept ClassicLRU's proven policy and geometry (32×64) but improved implementation details—early-exit on key match eliminates unnecessary bucket iterations, and 64-byte cache-line alignment reduces memory access latency. Result: 50% latency improvement on get operations and 33–50% on puts, with hit-rate preserved. This is the first variant to demonstrate that wide geometry's theoretical advantage (hit-rate) can be realized in practice through careful engineering.

- **Wide buckets are viable if latency is solved**: Cycle 0003 identified a latency crisis (p99 ≈300 ns) that made 32×64 geometry impractical. v003 proves the crisis was not geometric but implementational: address tail latencies through early-exit and cache alignment, and the geometry becomes a clear win—hit-rate +0.004 relative to ClassicLRU, latency returned to baseline levels.

- **Eviction optimization is a trap for high-occupancy buckets**: Both v001 and v002 optimized the eviction path, but at a bucket size (32–64 items), eviction is rare while hits are common. Spending per-hit cost to save infrequent eviction scans is a losing trade-off. v003 succeeds by optimizing the hot path (hit detection) instead.

## Open questions

- Is v003 a true Pareto win relative to ClassicLRU and all prior cycles? How does its fitness (hit-rate + latency) rank across the entire evolution history?
- Can v003's optimizations (early-exit, cache-line alignment) be backported to narrower bucket geometries, or are they specific to wide-bucket scanning efficiency?
- Does v003 maintain its advantage across different workload distributions, cache sizes, or multi-core contention scenarios, or is it optimized narrowly for the current Zipfian benchmark?
- Why does FIFO's victim selection require O(n) scan while ClassicLRU achieves O(1)? Could a data structure (e.g., linked list, heap, or O(1) min tracker) recover FIFO's zero-bookkeeping advantage without scan overhead?

## Continuation

The cohort demonstrates a critical insight: wide-bucket geometry is not inherently unviable, but requires micro-optimization to realize its hit-rate advantage without latency penalty. v003 is a turning point—it shows that the latency/hit-rate trade-off space has room for improvement through implementation cleverness rather than policy change.

Next cycle should: (1) confirm v003's fitness rank relative to the full evolution history and identify whether it is the current Pareto leader, (2) explore whether v003's optimizations (early-exit, cache-line alignment) generalize to other bucket sizes or narrow geometries, potentially unlocking further efficiency gains, (3) if v003 is the leader, probe its robustness across different workload distributions and cache sizes to identify its failure modes, or (4) if a larger policy innovation is desired, design approaches that prioritize hit-path efficiency—e.g., hybrid policies that fall back to ClassicLRU-like operations when micro-costs are high, or deferred/batched bookkeeping to amortize overhead.

Abandon further attempts to reduce per-hit bookkeeping overhead through policy isolation (v001, v002 both failed); the modern CPU memory hierarchy favors correct implementation of simple policies over complex policies with per-hit overhead.
