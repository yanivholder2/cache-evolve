# Variant v002 Insight

## Helped

- per-op-time (put): Frequency-distribution-based promotion thresholds reduce main queue evictions, lowering put latency by enabling faster small-queue evictions when the threshold is less permissive.

## Hurt

- hit-rate: The 75th percentile threshold was too conservative for this workload, resulting in fewer high-frequency items being promoted to the main queue, reducing cache hit rate by 0.58%.
- per-op-time (get): Maintaining the frequency sample moving window and computing percentiles during eviction adds latency to get operations, even under lazy evaluation.
- metadata-size: The VecDeque<u8> for frequency samples adds 40 bytes of per-instance overhead to track the 100-sample window.

## Takeaway

Frequency distribution provides a viable signal for adapting promotion thresholds and can significantly reduce put latency by changing eviction dynamics, but the 75th percentile is not the right cutoff for this workload. The approach sacrifices hit rate (the primary cache metric) for put performance, revealing that frequency percentile thresholds require calibration to workload characteristics. A lower percentile or adaptive percentile selection may recover the hit-rate loss.
