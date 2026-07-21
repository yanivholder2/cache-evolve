# Cycle 0009 Insight

## Cohort outcome

This cycle made its first targeted attempt on the get path, probing three distinct micro-optimisations: halving probe count via map consolidation, swapping to a lighter hasher, and eliminating redundant freq writes for saturated entries. All three are Winners; none moved get p50, and two produced net latency regressions — making this a fourth consecutive falsification cycle, now on the get-path axis.

## Patterns

- **The Zipfian hot set's L1 residency renders probe-count and hasher optimisations invisible**: Because the hot keys are accessed constantly, their map buckets stay in the highest cache level regardless of layout or hash algorithm. Halving probes (map merge) and substituting a simpler hash function therefore produce no measurable latency benefit — the cache miss being targeted simply does not occur on this workload.
- **Unconditional freq writes are load-bearing for eviction latency**: Skipping the freq write on saturated entries reduced the write-touch rate on those slots, allowing them to cool between `evict_main()` calls. When eviction subsequently scanned those freq entries, it paid a cache-miss penalty that was absent with the unconditional write. This is the inverse of the usual "fewer writes = faster" intuition: here, frequent re-dirtying keeps critical lines warm and is a net benefit.
- **SipHash is no slower than FNV-1a at this key size and cache scale**: The simpler hasher's theoretical advantage in CPU instruction count did not translate to wall-clock savings, suggesting SipHash's integer-optimised path is already at or near the throughput ceiling for the measured access rate.
- **Struct footprint shrank in two variants but fitness did not move**: Both the map-merge and the hasher-swap eliminated metadata bytes from the per-instance struct. These structural gains are real but orthogonal to the primary latency signal, confirming that per-instance size is not on the critical path at the tested cache capacity.

## Open questions

- With probe count, hash complexity, and write frequency all ruled out for get p50, is the get-path measurement itself the bottleneck — e.g., benchmark overhead, timer resolution, or workload construction — rather than any cache implementation detail?
- The Winners anomaly has now persisted across at least four consecutive cycles including this one; if the fitness threshold admits net-negative variants, does the current selection pressure meaningfully guide the search?

## Continuation

All three obvious get-path micro-optimisations — probe reduction, hasher replacement, write elimination — have been falsified in a single cohort. The L1 residency of the Zipfian hot set appears to make any approach targeting cache-miss reduction on the hot path structurally unreachable at this scale. The next cycle should either challenge the workload assumptions (e.g., probe the cold or medium-frequency tiers where cache misses do occur) or investigate whether the ghost-set membership test on every `put()` — the one per-operation cost that has not been directly targeted — is the remaining movable lever. Do not revisit map-merge vs. split layouts, custom hasher swaps, or write-skip guards for saturated entries; all three are exhausted on this workload.
