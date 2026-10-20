# Variant v003 Insight

## Helped

- metadata-size: Node is smaller than in the baseline. The hypothesis packed segment membership into a spare bit of an existing field, so the probation/protected split cost no extra bytes per entry. The whole cache struct did grow slightly, which fits the extra list head/tail and counters for the second segment.
- per-op-time: Both get and put latencies dropped. The likely cause is the smaller, denser Node, which gives better cache locality. The extra segment bookkeeping on a hit evidently cost less than that saving. I could not inspect the source, so this attribution is inferred from the hypothesis and the metrics.

## Hurt

- hit-rate: Hit rate on workload-a dropped slightly instead of rising as the hypothesis claimed. Promote-on-second-hit with an 80% protected cap and probation-head eviction did not beat a single LRU with the same sketch admission. The admission filter already supplies most of the scan resistance this trace needs, so the segmentation adds little.

## Takeaway

Segmenting the main LRU did not turn into a hit-rate gain on workload-a. Its benefit was the size and speed side effect of the compact 24-byte node layout. Scan resistance in the main region is therefore not the bottleneck behind the hit-rate plateau. A later variant could keep the compact layout and look for hit-rate elsewhere, for example in how the window or the admission decision is handled.
