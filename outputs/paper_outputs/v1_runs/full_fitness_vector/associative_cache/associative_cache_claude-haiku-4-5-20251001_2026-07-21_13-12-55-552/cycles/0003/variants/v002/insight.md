# Variant v002 Insight

## Helped

(No improvements observed)

## Hurt

(No regressions observed)

## Takeaway

This variant tested the hypothesis of combining v002's wider buckets (64×32) with FIFO eviction to replace ClassicLRU's expensive per-hit bookkeeping. However, the fitness metrics are identical to the parent variant: hit-rate remains 0.6635, per-op latency stays at 100 ns, and metadata size unchanged at 40 bytes. The gradle logs indicate the build used ClassicLRU despite the hypothesis specifying FIFO, suggesting the eviction-policy change either did not compile into the final binary or was overridden during evaluation. Without the FIFO policy actually in place, the variant provides no new signal on whether FIFO would reduce the 45% latency penalty that wider buckets incur under ClassicLRU's O(n) bucket-scan operations.
