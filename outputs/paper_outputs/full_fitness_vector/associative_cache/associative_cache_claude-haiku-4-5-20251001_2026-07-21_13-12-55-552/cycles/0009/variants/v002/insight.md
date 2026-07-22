# Variant v002 Insight

## Helped

- per-op-time (put): Morris counter eviction guidance reduces put latency by selecting better victims, improving throughput for insertion-heavy patterns.

## Hurt

- per-op-time (get): Probabilistic Morris counter increment logic in the hot get path (seed computation, mask, conditional check) adds ~31ns overhead per access, dominating the get p50 latency cost.

## Takeaway

Morris counters achieve the goal of fitting frequency metadata in 1 byte (leveraging struct padding), but the computation cost of probabilistic increment outweighs the benefit without hit-rate improvement. This workload lacks clear hot/cold access patterns that frequency-aware eviction exploits, making the latency penalty unjustified. Frequency guidance is viable only if both memory and compute costs stay negligible relative to the cache hit improvement it delivers.
