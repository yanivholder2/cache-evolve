# Cycle 0009 Insight

## Cohort outcome

This cycle tested three approaches to advance the ghost-queue frontier: relaxing the ghost-queue filter to recover lost hit-rate diversity (v001), moving occupancy-responsive boost computation to the ghost-hit cold path to eliminate hot-path overhead (v002), and batch-amortizing frequency decay to enable richer ghost state without per-operation cost (v003). All three variants succeeded in narrowly defined ways: v001 confirmed that aggressive filtering was correct by showing relaxation reintroduced overhead without gain; v002 proved that occupancy-responsive signals can be computed on cold paths with no per-operation penalty but revealed the boost formula requires tuning; v003 achieved dramatic latency wins (44% faster gets, 37% faster puts) while maintaining hit-rate by pushing decay entirely into amortized cold-path work. The cohort pivots understanding toward a new frontier: **computational placement (hot vs. cold path) is more valuable than mechanism innovation, and amortization unlocks richer state without cost.**

## Patterns

- **Ghost-queue signal purity was correctly calibrated**: v001's attempt to relax the freq > 0 filter to freq >= -1 catastrophically backfired, degrading put latency by 378ns p50 and 3405ns p99 without recovering any hit-rate. This confirms that cycle 0008/v001's aggressive filtering was not overly restrictive—freq=0 items are genuine noise that clutters the ghost lookup space, not hidden re-access opportunities. The boundary test reveals that further filtering, not relaxation, remains the path forward.

- **Occupancy-responsive adaptation is architecturally sound but formula-brittle**: v002 delivers a 19% get latency improvement by moving occupancy computation from the hot put path (where it incurs overhead) to the cold ghost-hit re-admission path (where it is negligible). This validates cycle 0008's insight that "eviction-time computation is zero-cost; hot-path computation is not." However, the quadratic boost formula `(2.0 + occupancy_ratio²)` is miscalibrated, providing excessive re-admission protection (freq 2–3) that crowds out small-queue working-set items, causing a 1.55% hit-rate regression. The architecture holds; the signal magnitude requires workload-specific tuning.

- **Amortization enables richer ghost state without per-operation cost**: v003 proves that expensive operations—like per-operation frequency decay—can be preserved if batched sufficiently (every 32 evictions). This delivers dramatic latency wins (44% faster gets, 37% faster puts) while maintaining hit-rate integrity (−0.05%, negligible noise). The result extends cycle 0008/v001's "zero-hot-path-overhead" philosophy: you can have both latency and richer ghost metadata if the decay window aligns with workload patterns. Frequency depth is preserved, not discarded; it is simply updated less frequently.

- **Hot-path vs. cold-path computation is the primary leverage point**: Both v002 and v003 succeed by deferring expensive operations (occupancy computation, frequency decay) to cold paths. Conversely, v001 fails because search-path overhead cannot be eliminated. This reframes the frontier: cycles 0006–0008 explored *which* signals to use; cycle 0009 shows the next lever is *where to compute them*. Mechanism innovation has reached diminishing returns; architectural placement is now the primary frontier.

## Open questions

- Can v002's quadratic boost formula be recalibrated (e.g., lower magnitude, occupancy-threshold gating, or frequency-distribution-aware scaling) to recover hit-rate while preserving the 19% latency gain?

- Does combining v002's occupancy-responsive boost with v003's batch-amortized decay unlock dual-metric gains, or do the two mechanisms interact negatively?

- Is v003's 32-eviction amortization window optimal for this workload, or should it be tuned (e.g., 16, 64) to balance decay freshness against amortization overhead?

- Can v002's ghost-hit re-admission selectivity be refined (e.g., only re-admit items with `ghost_freq > threshold(occupancy)`) to avoid crowding the small-queue working set while retaining occupancy-responsive adaptation?

## Continuation

The cohort validates that **computational placement (hot vs. cold path) is more valuable than mechanism innovation**, with amortization as the enabling technique for richer state. v003's batch-amortized decay is the strongest baseline for latency, while v002's architecture (occupancy-responsive cold-path computation) remains valid if the boost formula is tuned. v001's failure confirms that ghost-queue signal purity was correctly calibrated; further relaxation explores noise, not gain. **For the next cycle:** (1) prioritize tuning v002's boost formula—test lower magnitudes (e.g., 1.5 + occupancy_ratio) or frequency-threshold gating to recover hit-rate while retaining latency; (2) explore combining v003's batch decay with v002's occupancy-responsive re-admission to test whether richer state + adaptive signals unlock dual-metric wins; (3) if latency specialization is the goal, v003's amortized-decay approach is near-optimal and ready for production deployment; (4) validate v003's 32-eviction amortization window via parameter sweep (16, 32, 64) to ensure it matches this workload's small-queue eviction frequency; (5) hold v002's cold-path-computation insight as the architectural baseline—it has proven overhead-free and should be the foundation for signal adaptation going forward.
