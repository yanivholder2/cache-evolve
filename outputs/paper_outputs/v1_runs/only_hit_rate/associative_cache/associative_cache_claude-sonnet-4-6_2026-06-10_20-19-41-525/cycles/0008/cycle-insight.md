# Cycle 0008 Insight

## Cohort outcome

The cohort tested three distinct axes: removing mid-point insertion to save shift cost (v001), introducing an SLRU two-queue partition separating probationary and protected items (v002), and doubling the decay magnitude to collapse the lukewarm counter tier faster (v003). Notably, both v002 and v003 improved hit-rate — the first time in four consecutive cycles that any variant has done so — while v001 regressed. v003 was selected as winner.

## Patterns

- **The hit-rate design space is not saturated after all; structural separation of recency tiers helps.** SLRU's explicit probationary/protected split improved hit-rate by preventing scan traffic from displacing the confirmed working set — validating the long-standing suggestion that an adaptive two-queue scheme could break the local optimum. The co-encoding of position and counter in a single queue is no longer the only viable model.
- **Faster lukewarm demotion is a genuine, if narrow, lever.** Doubling the decay step converts the counter-2 tier directly to cold in one warm-lock event, modestly enlarging the eviction-candidate pool. The gain's small size indicates this tier is real but thin relative to the hot core.
- **The latency audit (v001) answered the hit-rate side but not the latency side.** Back-append removal of mid-point insertion confirmed that mid-point placement provides non-trivial eviction-ordering value — it was not merely cosmetic. However, per-op-time was again not captured, leaving the efficiency trade-off that has been deferred since cycle 0005 still unresolved.

## Open questions

- v002's SLRU design improved hit-rate but did not win the cycle; how does it compare quantitatively to v003, and is it the stronger long-term branch?
- Should SLRU segment sizing and promotion threshold be tuned, or is the current split geometry already near-optimal for workload-a?
- Per-op latency remains unmeasured across all variants; without it, the mid-point insertion trade-off and SLRU's additional bookkeeping cost cannot be evaluated.

## Continuation

SLRU is the most promising branch to pursue: it improved hit-rate through structural means rather than parametric tuning, opening a new evolutionary direction that the single-queue family could not reach. Next cycle should explore SLRU further — varying segment size ratios and promotion thresholds — while treating v003's doubled decay as the new parametric baseline. The latency measurement gap is urgent: SLRU introduces non-trivial per-operation bookkeeping, and without per-op-time data the efficiency case for or against it is blind. Do not revisit back-append as a hit-rate strategy unless the fitness vector is extended to include latency.
