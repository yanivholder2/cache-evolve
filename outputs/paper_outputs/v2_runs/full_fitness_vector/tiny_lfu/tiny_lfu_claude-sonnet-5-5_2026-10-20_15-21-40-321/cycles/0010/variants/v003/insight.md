# Variant v003 Insight

## Helped

- (none) No dimension improved. The adaptive window controller did not find a better window/main split on workload-a than the static one.

## Hurt

- hit-rate: Marginally below baseline. Hill-climbing on per-period hit ratio can only move the window around a split that was already near-optimal, so adaptation added no gain.
- metadata-size: The cache struct and the per-entry node both grew. The cache struct is consistent with the added controller state (counters, direction, step, target). The node growth is larger than the hypothesis intended and its source was not confirmed from the code.
- per-op-time: Both get and put got slower, and the tail latencies rose most. This is consistent with the hit-path counters, the period-end comparison, and the lazy window resizing in `put`. I did not inspect the code to attribute it to a specific one.

## Takeaway

On workload-a, adaptive window sizing did not pay for itself. It did not raise hit-rate, and it cost space and latency. This suggests the earlier flat results for static window sizes reflect a real plateau, not just untested adaptivity. Note: the overlay source was not readable when this was written, so the code-level attributions are inferred from the hypothesis and the fitness vectors.
