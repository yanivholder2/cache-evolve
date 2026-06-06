# Variant v003 Insight

## Helped

- hit-rate: Frequency tracking in the main zone identifies and protects hot items from eviction, improving retention of frequently-accessed entries on Zipfian workloads.

## Hurt

- per-op-time (get): Incrementing the frequency counter on every get operation for main-zone items adds a read-modify-write overhead, increasing latency by ~18%.
- per-op-time (put): Scanning up to 20 main-zone items to find a low-frequency candidate, with fallback to a second scan for LRU, adds O(n) work instead of the direct O(1) eviction, causing 57% latency increase at p50 and severe tail degradation (10x at p99).

## Takeaway

Frequency-selective eviction improves hit rate (+3%) on this Zipfian workload by protecting hot items, but the scanning-based implementation paid an unacceptable latency cost. Get operations became 18% slower and put operations 57–812% slower, revealing a fundamental trade-off: intelligent eviction requires either a faster data structure (e.g., heap, priority queue) or acceptance that O(n) eviction scans will dominate tail latency. V003 prioritized hit rate over latency robustness, contradicting the hypothesis's goal of "latency robustness."
