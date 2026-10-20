# Variant v003 Insight

## Helped

- None measured. The hit-rate dimension did not move on workload-a at size 2048, so the window did not deliver the recency-driven gain H03 targeted.

## Hurt

- per-op-time: The hypothesis adds a window LRU in front of the frequency gate, so every get that hits the window does a move-to-MRU and every put that fills the window pays an extra candidate comparison before the main-cache gate. Both paths add work over the baseline, and the get and put latency tails widened.
- metadata-size: No change in the reported per-entry node size. H03 expected one extra list anchor pair and one link per window entry, so the reported figure likely does not capture the window's bookkeeping. Treat this dimension as unverified rather than neutral.

## Takeaway

A small W-TinyLFU-style admission window did not raise hit rate on workload-a at 2048 entries, and it added per-operation latency, so the recency-protection idea does not pay off on this trace. Caveat: I could not read the overlay source in this session (no shell or directory listing was available), so this reading rests on the hypothesis text and the fitness vectors rather than the actual diff.
