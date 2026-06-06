# Variant v002 Insight

## Helped

- **hit-rate**: Granular frequency ranking within the bounded search window selects lower-frequency candidates over arbitrary positional selection, improving cache retention for workloads with non-binary frequency distributions within the scan window.

## Hurt

- **per-op-time (all metrics)**: Scanning the entire bounded window to track minimum frequency incurs additional overhead compared to v001's early-exit strategy on first `freq <= 1` match; the extra comparison and tracking logic adds latency across all operations, with put p99 suffering the most (38% regression) due to eviction path exposure.

## Takeaway

The shift from binary frequency thresholding to multi-level ranking improves hit rate by making smarter eviction decisions within the bounded window, but trades tail latency for better cache quality. The consistent latency increase across all operation types suggests the added complexity of scanning the full window outweighs v001's early-exit benefit when frequency distributions are mixed; the hypothesis succeeds on hit rate but exposes a latency cost not anticipated in the bounded-search strategy.
