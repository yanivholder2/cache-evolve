# Variant v003 Insight

## Helped

- hit-rate: The ghost history adds a +1 estimate bonus only to keys that were recently evicted and come back, and the bonus is applied before the victim comparison. That is a small additive nudge that only changes decisions when a newcomer sits right at the victim's estimate, so the gain is marginal rather than structural.

## Hurt

- metadata-size: The hypothesis adds a ghost FIFO ring of 64-bit hashes plus a membership set sized to the main-region capacity. That is extra per-cache bytes the baseline does not carry, but this variant's fitness vector reports no metadata-size entries, so the cost is expected rather than measured here.
- per-op-time: The ghost lookup on every `put()` and the push on every eviction add work to the admission path. This variant's vector reports no per-op-time entries, so the cost is expected rather than measured.

## Takeaway

Adding frequency credit for recently evicted keys, without touching the sketch's counters, is a low-risk change that barely moves hit-rate on workload-a at size 2048. The bonus is too small to do much on its own, so any future version that pursues ghost-based recovery would need a larger or more selective signal, and it should measure metadata-size and per-op-time, which this variant did not report.
