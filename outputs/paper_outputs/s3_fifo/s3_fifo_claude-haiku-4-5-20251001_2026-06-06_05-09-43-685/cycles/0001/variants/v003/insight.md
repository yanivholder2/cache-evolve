# Variant v003 Insight

## Hurt

- hit-rate: Expanding the small queue from 25% to 33% of capacity reduced hit rate, suggesting the larger admission window does not adequately compensate for the reduced main queue capacity and may delay promotion of hot items.

## Helped

- per-op-time: Increasing small queue size improves operation latency across both get (28–49% faster) and put (53–66% faster) operations, likely due to reduced contention and improved cache locality with a smaller main queue footprint.

## Takeaway

Increasing small queue capacity to 33% achieved the opposite effect: it worsened hit rate while unexpectedly improving operation latency. The hypothesis assumed a longer validation window would better identify hot items before main queue admission, but the reduced main queue (from 75% to 67%) appears to have more negative impact on retention of truly hot items than the admission filtering gains. The performance improvement suggests the change does reduce algorithmic overhead, but at a cost that outweighs the latency savings on this workload.
