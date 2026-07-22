# Variant v001 Insight

## Helped

- hit-rate: Amortizing buffer updates to every 32nd insertion preserved the learned insertion-positioning benefit, maintaining the 0.665 hit-rate and confirming that less frequent tracking retains accuracy for biasing placements toward reuse-heavy phases.

## Hurt

- per-op-time: Concentrating deferred buffer work into every 32nd operation created latency spikes; while p50 get time remained flat at 100ns, p99 doubled from 100ns to 200ns, indicating that amortization boundaries accumulate overhead into intermittent expensive operations rather than smoothing it across all accesses.
- metadata-size: The insertion_count field added 16 bytes to the cache struct (64→80 bytes) to implement the amortization interval tracking.

## Takeaway

Amortizing insertion-bias buffer updates every K operations preserves hit-rate parity but trades uniform latency for bursty behavior. While median latency remains unchanged, tail latency degradation (p99 get +100%) reveals that deferral consolidates overhead into periodic spikes rather than distributing it smoothly, undercutting the hypothesis that amortization would reduce per-operation cost from 40% to 1–2%.
