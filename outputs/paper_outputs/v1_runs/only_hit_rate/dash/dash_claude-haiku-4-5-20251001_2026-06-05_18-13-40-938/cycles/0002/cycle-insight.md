# Cycle 0002 Insight

## Cohort outcome

This cycle tested three strategies for accelerating stash-hit promotion through the tier hierarchy. The winner (v001) validated that combining direct tier-skipping with MRU positioning yields measurable hit-rate improvement; a second successful variant (v002) showed that adaptive occupancy-based strategies can also improve hit rate. A third strategy attempting unconditional backpressure relief via occupancy thresholds hurt hit rate by promoting non-hot items, confirming that occupancy alone is not a sufficient signal for acceleration.

## Patterns

- **Fitness signals gate acceleration**: Both successful variants preserved item hotness or recency signals before bypassing tiers. v001 promoted only stash hits (proven survivors); v002 also promoted only stash hits, adding an occupancy check. v003's failure—promoting any target item when stash > 60%—showed that occupancy-driven relief without a fitness check creates churn rather than relief.
- **Threshold sensitivity in occupancy-adaptive strategies**: v002's 75% occupancy threshold on stash-hit promotion succeeded, while v003's 60% threshold on unconditioned target promotion failed. The threshold alone doesn't determine success; the fitness check (stash hit vs. target hit) is more critical than the occupancy level.
- **Two paths to stash-hit acceleration converged**: v001 (deterministic tier-skip) and v002 (occupancy-adaptive tier-skip) both work, suggesting that the core insight—fast-tracking stash hits to probing—is robust across at least these two implementation strategies.
- **MRU positioning remains relevant**: v001's explicit use of Back insertion for MRU placement validates the cycle 0001 lesson that position within a tier matters; v002 implicitly inherited this by copying v001's logic.

## Open questions

- Why was v001 selected as winner over v002, despite both improving hit rate? (Simplicity, consistency, magnitude of gain, or tail behavior?)
- Does v001's deterministic acceleration degrade under sustained high stash occupancy, where v002's adaptive switch might excel?
- Is there an occupancy threshold where adaptive strategies (like v002) outperform deterministic ones (like v001), or are they equivalent under this workload?
- Could v002's adaptive logic be applied to v001 (occupancy-aware MRU positioning) to preserve determinism while handling load spikes?

## Continuation

Keep v001 as the new baseline—it's simple, validated, and improves hit rate. Investigate whether v002 handles high-pressure scenarios better by testing both strategies under sustained cache congestion; if v002 wins at peak load, consider an adaptive hybrid for production. Definitively drop occupancy-as-a-relief-valve approaches unless they are gated by hotness checks; the v003 failure confirms occupancy is not a fitness signal. In the next cycle, focus on understanding whether the deterministic v001 or adaptive v002 generalizes better, and whether combining both (occupancy-aware stash-hit acceleration) would amplify gains.
