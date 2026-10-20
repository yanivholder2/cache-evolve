# Variant v001 Insight

## Helped

- **metadata-size**: Replacing `insertion_bias: f64` with `insertion_bias: u8` (scale 0–15) reduced the AssociativeCache struct footprint from 80 to 72 bytes, reclaiming 8 bytes of memory overhead that the parent's characterization tracking added.

- **per-op-time (get p99)**: The bit-shift operation `(insertion_bias as usize * len) >> 4` in `compute_insertion_position()` improved get p99 latency by 100 ns compared to parent's floating-point multiplication, suggesting better CPU cache utilization from the smaller u8 field footprint.

## Hurt

- **hit-rate**: The variant regressed from 0.718 to 0.7001, losing 1.8 percentage points of the parent's 8.1% gain. Quantizing the continuous insertion bias (0.0–1.0) to a discrete 0–15 scale introduces precision loss; workload characterization computes a hit ratio that maps poorly onto the 16-level quantization (e.g., 0.55 → 8/15 ≈ 0.533), shifting insertion positioning away from the optimal adaptive bias.

- **per-op-time (put)**: Latency improvements did not materialize for put operations (p50 and p99 remained unchanged), despite the hypothesis's claim that removing floating-point arithmetic from the hot path would reduce per-put cost.

## Takeaway

The variant demonstrates that integer quantization, while reducing memory overhead, comes at the cost of algorithmic precision. The hypothesis correctly identified floating-point multiplication as a latency concern and successfully optimized the hot path with integer bit-shift arithmetic; however, quantizing the learned insertion bias to 8 bits destroyed the subtlety of the workload characterization, causing hit-rate to regress below the baseline. This teaches that not all optimizations can be compressed without loss—biases tuned by characterization require sufficient precision to remain effective, and naive type conversion can erase the benefit it was meant to optimize.
