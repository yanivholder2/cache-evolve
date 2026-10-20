# Cross-Cycle Insights

## Recurring wins

- **Sketch width expansion (collision reduction)**: Cycle 0001's doubling of width improved both hit-rate and per-operation latency simultaneously, becoming the reference baseline (depth-4, width-8) for all subsequent cycles. The mechanism—reducing hash collisions within rows—justifies memory cost on Zipfian workloads with multiplicative ROI.

- **Moderate depth reduction (4→3) yields clean trade**: Cycle 0002 extracted 25% per-operation latency gain (132→90 ns get) for <0.3% hit-rate loss. This baseline remained stable and was reused in cycles 0003–0004, validating that within-row collision density (width), not row depth, is the bottleneck.

- **Temporal freshness via sketch decay at 500k interval**: Cycle 0003 broke the latency-accuracy tradeoff—the only variant to improve both metrics simultaneously. Periodic frequency-estimate reset (decay every ~500k operations) improved hit-rate and reduced per-operation latency on depth-3 width-4 baseline, indicating a bottleneck orthogonal to collision density.

## Recurring failures

- **Admission threshold micro-tuning is workload-overfit with no gradient**: Cycles 0002 (v001: relaxed >=) and 0003 (v002: offset -1) both degraded hit-rate by admitting marginal items that displaced higher-value residents. The strict `incoming_freq > victim_freq` condition encodes domain-specific temperature distribution with no smooth tuning range; relaxation consistently backfires.

- **Width expansion hits diminishing returns and cost barriers**: Cycle 0001's 8x expansion was high-ROI; cycle 0003 v001 repeated the expansion (4→8) on depth-3 baseline and paid 60% latency surge (90→144 ns) and 2x memory cost for only +0.6% hit-rate. Cost-benefit ratio is cache-size and geometry specific, not universal.

- **Sketch decay is brittle across parameter variations**: Cycle 0004's attempts to re-tune decay frequency failed across the board—250k interval (v001) and 1M interval (v002) both regressed, and pairing 500k decay with width-8 geometry (v003) created catastrophic tail-latency spikes (p99 get: 154→243 ns). Cycle 0003's breakthrough appears to be a narrow local optimum confined to depth-3 width-4, not a composable lever.

## Open questions

- **Was cycle 0003's 500k decay genuinely reproducible, or a statistical artifact?** All re-tuning and recomposition attempts in cycle 0004 failed. If the breakthrough was real, what workload or geometry property makes 500k optimal? If not, the finding was a false positive.

- **Is sketch decay compatible with different sketch geometries?** Decay was tested only on depth-3 width-4 (won) and depth-3 width-8 (catastrophic regression). Does decay require a specific "Goldilocks" geometry, or can implementation be made more robust (e.g., lazy resets, probabilistic sampling to reduce overhead)?

- **What orthogonal levers remain beyond sketch parameters?** Cycles 0001–0004 exhausted sketch geometry (width, depth, decay interval) and admission threshold tuning. Untested dimensions: eviction policy variations (LFU vs heuristic victim selection), hash function design, frequency aging strategies outside periodic reset, adaptive parameter schedules tied to workload phases or cache utilization.

- **How do findings scale to different workload characteristics?** All tuning is on a single Zipfian trace at one cache size. Depth-3 width-4 may be overfit to this specific workload; decay's phase-responsiveness benefit is unknown on bursty or phase-shifting traces; width-4 may not be optimal at larger cache capacities.

## Per-policy notes

- **Sketch geometry (width, depth, decay)**: Width expansion is high-ROI for collision reduction (cycle 0001 baseline, width 8 stable), depth-3 is a stable latency-accuracy trade (25% latency for <0.3% hit-rate), decay at 500k improved both metrics on depth-3 width-4 but shows brittle behavior across all parameter variations (cycles 0003–0004). Geometry tuning appears to have plateaued at depth-3 width-4 ± decay.

- **Admission policy (threshold and selectivity)**: Strict `>` threshold is workload-optimized with no smooth tuning gradient. All relaxation attempts (cycles 0002–0003) degraded hit-rate; the threshold encodes temperature distribution knowledge that marginal adjustments cannot improve. This policy dimension is at local optimum and resists further micro-tuning.
