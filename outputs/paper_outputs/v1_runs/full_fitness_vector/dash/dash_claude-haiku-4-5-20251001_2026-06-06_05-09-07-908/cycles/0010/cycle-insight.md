# Cycle 0010 Insight

## Cohort outcome

This cycle tested three distinct refinement strategies: further divisor tuning on the proven 75/25 geometry (v001), composition of frequency-aware eviction with that same geometry (v002), and a new mechanism leveraging frequency information for demotion cascade gating (v003). Only v003 succeeded. The standout is v003's frequency-aware demotion gating, which introduces a mechanically orthogonal optimization axis by preventing cold items from entering target tier unnecessarily, delivering dramatic latency improvements (~57% on get, ~31% on put) with minimal hit-rate trade-off (0.14% loss). v001 and v002 confirm that further tuning of geometry and promotion parameters faces diminishing returns; new independent mechanisms are required for additional gains.

## Patterns

- **Frequency information is valuable for demotion gating, not eviction candidate selection**: v002 shows that frequency-aware eviction (tracking both recency and frequency for composite ranking) adds overhead without improving hit-rate or latency when applied to v003's proven 75/25 geometry. v003 shows that frequency-aware demotion gating—using frequency thresholds to prevent cold items from entering target tier—delivers massive latency gains. The distinction is mechanical: frequency helps *where items go* (demotion routing), not *which items to evict among candidates* (eviction ranking). This resolves the frequency-exploration branch opened in cycle 0008.

- **Geometry-and-promotion optimization has reached a local plateau**: v001's failure to improve latency by further reducing selectivity (divisor 4→3 on 75/25) confirms that cycles 0007–0009 achieved a stable equilibrium on this parameter space. The 75/25 geometry with divisor 4 is locally optimal; moving the selectivity dial degrades or stalls performance. This contrasts sharply with v003's success via a new mechanism, signaling that additional gains require new axes, not further tuning of existing dials.

- **New independent mechanisms outperform parameter refinement on saturated dimensions**: v001 attempted optimization within the existing (geometry, divisor) space and failed; v003 introduced a new orthogonal axis (demotion cascade gating) and succeeded dramatically. This reinforces the cycle 0009 insight that the optimization space has matured beyond incremental parameter tuning. The path forward is discovering new independent mechanisms, not refining existing ones.

- **Cascading demotion behavior is a critical performance lever**: v003's mechanism addresses a different part of the cache pipeline than promotion gating or geometry allocation. By gating the "one-more-chance" demotion based on frequency (lfu_counter < 2 → bypass target, go directly to stash), the variant reduces cascade depth and contention on hot items without sacrificing hot-item retention. This demonstrates that tier-crossing heuristics—specifically, conditional cascading policies—can deliver outsized latency improvements.

- **Latency and hit-rate trade-offs are now smaller and more selective**: v003's 0.14% hit-rate loss for ~57% get-latency gain represents an excellent trade-off ratio compared to earlier cycles. The separation between latency and hit-rate optimization is now more surgical: demotion gating targets eviction efficiency without wholesale hit-rate sacrifice.

## Open questions

- Does v003's frequency-aware demotion gating succeed across all geometries (baseline, mid-range, aggressive), or is its benefit localized to baseline geometry?
- Can v003's demotion gating mechanism be combined with the proven 75/25 geometry and divisor 4 formula from cycle 0009's v003 to compound latency improvements?
- Are there other unexplored cascading or tier-crossing heuristics (e.g., adaptive second-chance thresholds, frequency-aware promotion cascading) that could deliver similar or complementary gains?

## Continuation

Divisor-based promotion tuning on fixed geometries has saturated; v001's failure confirms diminishing returns on further refinement. Frequency-aware eviction candidate selection is confirmed ineffective—close that exploration axis permanently. Frequency-aware demotion cascade gating (v003) is the breakthrough mechanism and should be prioritized: test its efficacy across the full geometry spectrum (especially 75/25 from cycle 0009) and explore composition with proven geometry-divisor pairs. If demotion gating composes well with 75/25 geometry and divisor 4, that combination becomes the new optimized baseline. If not, adopt v003's baseline+demotion variant as standard and shift focus entirely away from geometry and promotion tuning toward new independent axes: tier-crossing heuristics, adaptive cascade policies, bucket-sizing dynamics, or promotion-chain optimization. Do not pursue further divisor refinement or frequency-aware eviction variants.
