# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

This cohort tested three distinct improvements to ClassicLRU: a frequency-aware eviction policy (LFU), a refined bucket structure, and an asymptotically faster removal mechanism (Swap). All three variants degraded latency performance relative to baseline, and none improved hit-rate on the Zipfian workload. The cohort demonstrates that theoretical advantages—frequency awareness, finer-grained bucket isolation, O(1) removal—do not survive contact with implementation overhead and memory access patterns in practice.

## Patterns

- **Latency regressed uniformly**: LFU, bucket repartitioning, and Swap each introduced measurable per-operation costs (ranging from 45% to 3× slowdowns), indicating that any added bookkeeping or structural change incurs penalties that the baseline already avoids.
- **Hit-rate was flat or worsened**: Two variants maintained 0.6611 while one degraded to 0.6568, suggesting that ClassicLRU's simple recency-based eviction already aligns well with Zipfian access patterns and that the workload does not reward frequency tracking or fine-grained bucket segregation.
- **Theory vs. practice gap widest for Swap**: The policy's asymptotic O(1) removal advantage (vs. ClassicLRU's O(n)) inverted at the benchmark scale, implying that constant factors—cache coherency, memory latency, swap chain overhead—dominate actual performance for this geometry.
- **Baseline stability**: The consistent hit-rate of 0.6611 across variants suggests the workload's access distribution is captured by recency alone, making the ClassicLRU baseline robust for this scenario.

## Open questions

- Would batched or vectorized updates to frequency counters (or swap chains) recover performance while retaining the semantic benefits of LFU or Swap?
- Does ClassicLRU's dominance hold across other access distributions, or is the Zipfian pattern particularly amenable to recency-only eviction?
- Could adaptive or hybrid policies dynamically switch between eviction strategies depending on observed access patterns or hit-rate trends?

## Continuation

The cohort's uniform regression against ClassicLRU suggests this baseline is sound for the Zipfian workload and current cache geometry. Rather than pursuing further policy refinements in isolation, next cycle should: (1) test ClassicLRU's robustness across different workload distributions or cache sizes to identify scenarios where it breaks, (2) explore implementation-level optimizations (prefetching, batching, vectorization) that might reduce per-access overhead without changing policy, or (3) shift focus to scenarios (non-Zipfian access, cache contention, multi-core effects) where simpler eviction may falter. If frequency or finer buckets are revisited, prioritize measurement and optimization of implementation overhead as the primary constraint, not policy semantics.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cohort tested three orthogonal approaches to improve upon ClassicLRU: replacing item-movement bookkeeping with timestamp updates (TimestampLRU), minimizing bookkeeping entirely (FIFO), and consolidating cache geometry into wider, denser buckets (64 × 32). All three variants regressed on per-operation latency by 40–45%, while hit-rate either held steady or improved modestly (+0.24%). No variant achieved a fitness win over the baseline, revealing that ClassicLRU is not just a reasonable policy for this workload but a particulary well-optimized implementation of one.

## Patterns

- **Latency regressed uniformly despite different mechanisms**: Whether changing eviction policy (TimestampLRU, FIFO) or cache geometry (wider buckets), all variants slowed by ~40–45% on per-op time, suggesting ClassicLRU's vector operations on small buckets (16 items) have unusually good cache locality or compiler optimization that alternatives could not match.
- **Hit-rate insensitivity to policy details**: FIFO maintained 0.6611 hit-rate despite dropping recency awareness entirely, confirming that Zipfian access patterns alone can sustain high hit-rate without explicit recency tracking. TimestampLRU also held 0.6611, showing the bookkeeping method is orthogonal to hit-rate. Only geometry change (V002) moved hit-rate (+0.24%), but at a larger latency cost than policy changes.
- **Geometry trade-off: hash table vs. bucket scan**: Wider buckets (64 × 32) improved cache density and hit-rate but increased the cost of sequential intra-bucket searches, suggesting that for this workload, the latency penalty of scanning longer lists outweighs the benefit of fewer hash table lookups and better memory packing.
- **Theory-practice inversion deepens**: V001 (TimestampLRU) pursued O(1) timestamping vs. ClassicLRU's O(n) vector ops, yet achieved worse latency. This repeats the pattern from cycle 0001's Swap variant: asymptotic complexity gains evaporate under cache-level micro-architecture and implementation overhead.

## Open questions

- Why does ClassicLRU's O(n) vector manipulation (remove + append on 16-item vectors) outperform O(1) timestamp updates or zero-cost FIFO hits? Is the answer cache line coherency, branch prediction stability, or compiler loop optimization?
- Does the current geometry (128 × 16) happen to be locally optimal for this Zipfian workload and benchmark scale, or is it more broadly robust?
- Would batched or vectorized bucket scans recover the geometry advantage, or is the per-item cost of scanning inherently the bottleneck?

## Continuation

ClassicLRU appears not simply correct but finely optimized for this workload and hardware—attempts to improve hit-rate, reduce bookkeeping, or tune geometry all degraded latency without achieving a net fitness gain. Rather than continuing policy or geometry variations, the next cycle should: (1) understand *why* ClassicLRU is fast (profile micro-architectural effects, examine compiled code patterns), (2) test ClassicLRU's robustness across different workload distributions, cache sizes, or multi-core scenarios to identify conditions where it breaks, or (3) explore implementation-level refinements (prefetching, cache-aligned layouts, vectorization) that preserve ClassicLRU's performance profile while addressing potential bottlenecks outside the policy itself. If policy is revisited, focus on hybrid or adaptive strategies that can fall back to ClassicLRU's proven efficiency rather than replacement.


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
