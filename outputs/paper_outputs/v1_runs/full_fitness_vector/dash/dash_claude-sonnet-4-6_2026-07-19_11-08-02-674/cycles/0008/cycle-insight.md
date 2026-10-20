# Cycle 0008 Insight

## Cohort outcome

This cycle explored three independent directions: bit-packed clock/frequency decoupling on the target tier (v001), frequency-gated stash bypass to probing (v002), and cold-preferring stash eviction on a prior base (v003). All three variants won — not because they improved hit-rate, but because none introduced severe regressions; v003 produced a meaningful latency improvement while v001 and v002 were neutral-to-slight-regression and fully neutral, respectively.

## Patterns

- **The CLOCK_BIT fix is correct but insufficient.** v001 resolved the counter-corruption bug from Cycle 0007 by encoding the clock signal in the high bit of `lfu_counter`, preserving the frequency gate. However, because CLOCK_BIT is set on *every* non-hottest target access, nearly all resident target items are perpetually "warm" — leaving the clock with no discriminating power over what LFU-minimum eviction already provides on this counter-rich base. The bit-packing technique works as designed; the problem is that the warmth signal fires too broadly.
- **The stash tier is a consistently weak lever for hit-rate on this workload.** Both v002 (warm-stash bypass to probing) and v003 (cold-preferring stash eviction) left hit-rate unchanged. The stash represents too small a fraction of total cache footprint, and the conditions that move items into stash (target-full cascades) rarely produce items with high frequency counts, so gate-based stash interventions fire too infrequently to shift the aggregate miss curve.
- **Stash cold eviction is latency-cheap and amortized safely.** v003's scan runs only at full-stash-bucket insertions — far less often than target-tier scans — confirming that the stash tier can absorb frequency-aware eviction logic without a tail-latency cost. This disproves the concern that any O(n) stash scan would replicate Cycle 0005's latency regression.

## Open questions

- Can CLOCK_BIT be set only at a higher frequency threshold (e.g., `lfu_counter >= 3`) rather than on every non-hottest access, so that the clock retains discriminating power without corrupting the promotion gate?
- With eviction policy (target), promotion gating, probing re-entry, stash eviction, and stash bypass all explored, what structural axes remain — alternative tier layouts, adaptive thresholds, or workload-specific tuning — that could move hit-rate?
- Would stash cold eviction (v003) contribute on a workload or cache size where the stash tier holds a larger share of total capacity?

## Continuation

The clock integration problem is now well-characterized: the warmth signal needs to be sparse, but the existing promotion machinery makes it dense. Next cycle should test a higher CLOCK_BIT threshold (set only when `lfu_counter` crosses a second boundary, not on every increment) or replace CLOCK_BIT with a genuinely independent boolean "chance bit" that is set only when an item is rescued from eviction, not on ordinary accesses. The stash tier has been explored from multiple angles without a hit-rate payoff and should be deprioritized. v003 is the cleanest forward base (latency improvement, no regressions on its parent lineage), and v002 is safe but adds nothing; future work should build on v003 if targeting latency, or revisit the clock idea with a sparser warmth-set condition if targeting hit-rate.
