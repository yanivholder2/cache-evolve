# Cross-Cycle Insights

## Recurring wins

- **Frequency estimation accuracy is the foundational lever**: Cycle 0001's width expansion and cycle 0002's validation that poor frequency estimates (v001 in 0002, when paired with permissive admission) drive worse outcomes confirm that collision reduction in the sketch pays multiplicative returns across hit-rate and latency.
- **Moderate depth reduction is a stable trade-off**: Cycle 0002's depth 3 achieved 25% per-operation latency gain for <0.3% hit-rate loss, validating cycle 0001's suspicion that the lower bound on sketch depth sits around 3 layers without catastrophic accuracy loss.
- **Width-based optimization has higher ROI than depth exploration**: Cycle 0001's width increase improved both metrics simultaneously; cycle 0002's depth expansion (v003) showed diminishing returns, establishing that hash-row collisions are not the primary bottleneck.

## Recurring failures

- **Relaxing the admission threshold without isolation**: Cycle 0001's v003 traded hit-rate for throughput (an orthogonal trade), but cycle 0002's v001 combined width expansion + relaxed admission (>=) and hit-rate declined. This suggests the strict > threshold encodes workload-specific domain knowledge about temperature distribution that marginal frequency improvements cannot override.
- **Expanding sketch depth beyond 4 layers**: Cycle 0002's v003 expanded to 6 layers, yielding only +0.33% hit-rate improvement while incurring 11% latency cost and 50% memory overhead—a clear signal that collision reduction has saturated as a lever.
- **Prioritizing per-operation latency over frequency accuracy**: Cycle 0001's v002 reduced depth to improve p50 but degraded hit-rate and p99, revealing that tail latency and operational outcomes depend more on admission decision quality than raw hash computation speed.

## Open questions

- **Can depth 3 + increased width recover hit-rate losses?** Cycle 0002's v002 proved depth 3 is latency-stable; pairing it with width expansion (similar to cycle 0001's v001) might compound benefits and answer whether the two levers are truly orthogonal.
- **What is the gradient of the admission threshold?** Does an intermediate condition (e.g., incoming_freq >= victim_freq - 1) exist between strict > and permissive >=, preserving admission selectivity without the hit-rate cliff observed in cycle 0002's v001?
- **Do sketch dimensions and admission policies adapt to workload skew?** Both cycles assume a fixed Zipfian trace; unclear whether the width-over-depth preference and strict threshold remain dominant on denser or sparser access patterns.

## Per-policy notes

- **Admission policy (LFU threshold)**: The strict incoming_freq > victim_freq rule is load-bearing and not easily overridden by improved frequency estimation. Cycle 0002's v001 demonstrated that relaxing to >= admitted items with borderline frequencies, degrading hit-rate despite a wider sketch. The threshold appears overfit to this workload's temperature distribution and warrants isolated tuning (independent of geometry changes) to understand its gradient.
- **Sketch geometry (width vs. depth)**: Width and depth are non-interchangeable levers. Width expansion (cycle 0001) delivered multiplicative returns; depth expansion (cycle 0002's v003) shows diminishing returns. Moderate depth reduction (cycle 0002's v002) is viable and aligns with cycle 0001's hypothesis that depth 3–4 is the sweet spot, freeing memory and compute for width increases.
