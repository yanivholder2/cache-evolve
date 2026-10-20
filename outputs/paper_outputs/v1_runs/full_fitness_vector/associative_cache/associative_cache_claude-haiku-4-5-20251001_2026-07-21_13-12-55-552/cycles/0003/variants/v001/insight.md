# Variant v001 Insight

## Hurt

- **hit-rate**: Narrower buckets (256×8) reduce collision tolerance on Zipfian workload, causing 0.43% hit-rate degradation despite preserved total capacity.
- **per-op-time** (get p50): 45% latency regression (69ns → 100ns) contradicts the hypothesis that reduced scan depth would improve latency. Hash-function distribution overhead and memory-layout effects of wider bucket counts likely dominate over savings from shallower scans.

## Helped

(none)

## Takeaway

Narrowing buckets under ClassicLRU does not improve latency as predicted. The hypothesis assumed that reducing scan depth (from 16 to 8 items) would deliver measurable latency gains, but the actual penalty—likely from hash distribution costs and memory-access patterns across 256 buckets—outweighs any benefit. Narrower geometry is a liability for both throughput and hit-rate on this workload.
