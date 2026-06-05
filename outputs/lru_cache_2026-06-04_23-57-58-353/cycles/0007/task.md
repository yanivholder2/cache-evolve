# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007

## Winners

- v003

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants extended the cycle-3 frequency-demotion baseline in different directions — scan-fraction widening, ghost-window expansion, and frequency-aware eviction in probationary. Only the probationary extension (v003) won, by ensuring that the freq counter preserved across demotion actively shields recently-warm entries at both eviction sites.

## Patterns

- The frequency counter and counter-halving-on-demotion design have been the load-bearing mechanism since cycle 3, but it was applied asymmetrically: protected-segment demotion respected frequency, while probationary eviction discarded it via blind LRU. v003 closed that gap — the signal survived the segment boundary but was being thrown away at the next eviction decision.
- Scan-fraction tuning above 1/4 in the protected segment is now confirmed to offer negligible returns on this workload. Going from 1/4 to 1/3 moved hit rate only marginally, indicating the frequency variation near the LRU end of protected is mostly captured by the narrower window already.
- Ghost-window expansion to full capacity regressed hit rate: reuse events at distances between probationary capacity and total capacity on this trace are not reliable locality signals, and fast-pathing those keys into protected pollutes it. The effective ceiling for the ghost window sits well below total capacity, and the ghost-window expansion thread is now largely exhausted.

## Open questions

- Is the probationary LRU-half scan fraction robust, or should it be adaptive? A workload with longer reuse distances or a more skewed access distribution might need a different fraction at the probationary eviction site.
- With frequency awareness now applied at both eviction sites, is there any remaining structural gap, or have the major policy improvements been captured? A direct comparison against a wider workload suite would clarify how general these gains are.
- Could an intermediate ghost window (e.g., 50% of total capacity, between probationary capacity and the failed full expansion) recover some cross-residency signal without the admission-quality problem seen at full capacity?

## Continuation

Next cycle should take v003 as the new baseline: two-segment SLRU, counter-halving demotion, frequency-aware eviction scanning the LRU quarter of protected and the LRU half of probationary. Ghost-window expansion can be deprioritized — the full-ceiling test failed and intermediate values are unlikely to move the needle meaningfully on this trace. The most productive direction is probing the probationary scan fraction (is 1/2 near optimal, or does a smaller fraction suffice?), and testing whether either or both scan fractions should vary proportionally with observed reuse-distance patterns rather than being fixed.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants probed the cycle-4 frequency-aware baseline from different angles — scan-fraction narrowing, counter inflation control, and admission gating — and none produced a hit-rate gain. The winner (v002) advanced on neutrality: the frequency cap introduced no regression while the other two variants hurt performance, making it the best performer in a cohort that collectively confirmed the current configuration is near a local optimum on this workload.

## Patterns

- The probationary half-scan is now confirmed as genuinely necessary, not merely conservative. Frequency variation among probationary entries — cold newcomers mixed with demoted entries carrying halved counters spanning a wider range — is spread throughout the list, not compressed near the LRU head. The quarter-scan that suffices for protected (where all entries have at least one access and frequency variation near the LRU end is compressed) does not transfer to probationary.
- Counter inflation is not an active problem on this workload. The halving-on-demotion mechanism already prevents runaway accumulation in practice: entries must remain in protected without ever being demoted for counters to inflate far above the cap, and that residency pattern does not occur at this cache size on workload-a. The MAX_FREQ=8 cap is structurally sound as a guardrail but targets a pathology the trace does not exhibit.
- The frequency-aware eviction scan introduced in cycle 4 already provides sufficient cold-traffic filtering in probationary. A two-hit promotion gate is redundant protection — the entries that survive in probationary are generally worth promoting immediately, and the extra exposure window costs more hot-key residency than the one-hit-wonder reduction saves.
- Two consecutive cohorts (cycles 4 and 5) have produced wins that are structural closures rather than fresh gains: cycle 4 closed the asymmetry between eviction sites; cycle 5 confirmed that three further refinements of the resulting design produce no improvement. This is a convergence signal for the current policy on this workload.

## Open questions

- Is the policy now at a local optimum for workload-a, or are there unexplored structural directions (e.g., adaptive scan fractions tied to observed hit-rate feedback, segment-ratio tuning, or load-sensitive demotion) that could break out of the current plateau?
- Would the frequency cap (v002's change) matter on a workload with longer-lived hotspots where continuous protected residency without demotion is more common?
- Is widening the evaluation to a second workload the most informative next step — to distinguish genuine convergence from workload-specific saturation?

## Continuation

Next cycle should carry v002 forward as the new baseline (the MAX_FREQ=8 cap is a harmless guardrail worth keeping). The probationary half-scan and immediate single-hit promotion are both settled — neither narrowing the scan nor adding a promotion gate is productive on this workload. With three scan-fraction and admission variants now exhausted, the highest-value direction is either a genuinely new structural axis (adaptive policy parameters, segment-ratio exploration beyond the previously tested range, or a different eviction philosophy) or explicit multi-workload evaluation to determine whether the policy has converged broadly or only on workload-a.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort revisited three previously-identified threads — a referenced-bit tie-breaker in the demotion scan, an intermediate ghost window at half of total capacity, and logical-clock frequency decay at victim-selection time — and none produced a hit-rate gain. The winner (v001) advanced on neutrality for the second consecutive cycle, confirming the policy is at a workload-a local optimum.

## Patterns

- The referenced-bit tie-breaker (v001) is structurally sound and avoids the cycle-3 position-freeze failure, but it targets a collision pattern — equal-frequency candidates where one is recently referenced — that this trace does not produce at the narrow LRU-quarter scan window. The mechanism is inert rather than harmful.
- The ghost window is now exhaustively mapped across the meaningful range: probationary capacity is too narrow to matter, half-capacity regresses, full capacity (cycle 4) regressed harder. Reuse events at distances between probationary capacity and half-total-capacity are cold traffic on this workload, not deferred-warm traffic, and the ghost mechanism cannot be rescued by window sizing alone.
- Logical-clock decay at a fixed half-life (v003) is acutely workload-sensitive. The value of the existing frequency signal comes from distinguishing demoted-from-protected entries from cold newcomers; a half-life shorter than the typical inter-access gap on this trace collapses that distinction and eliminates the scan's ability to protect warm entries — the same problem it was meant to sharpen. Any fixed decay constant is unlikely to generalize without being tuned to the workload's reuse-distance distribution.
- Two consecutive cohorts (cycles 5 and 6) have produced winners that advance on neutrality rather than gain. This is a convergence signal: the main structural gaps in the policy have been closed, and the remaining unexplored axes either require workload-specific tuning or are genuinely unproductive on workload-a.

## Open questions

- Is there any fixed-rate decay constant (e.g., half-life at 2048 or 4096 ops) that preserves the frequency distinction on workload-a, or does the inter-access distribution make any single constant wrong?
- With ghost admission, scan fractions, promotion gating, frequency caps, referenced bits, and decay all exhausted, are there structural eviction philosophies not yet tested — adaptive segment sizing, ARC-style balance feedback — that could break out of the current plateau?
- Is the policy broadly strong, or specifically tuned to workload-a? Multi-workload evaluation is now the clearest way to answer this.

## Continuation

Next cycle should treat the current policy (frequency-aware demotion and eviction, counter halving on demotion, MAX_FREQ cap, optional referenced-bit tie-breaker) as a stable baseline and pivot away from parameter tuning on workload-a. Ghost window and fixed-rate decay are both exhausted as productive directions. The highest-value move is either introducing a second workload to test transferability of the gains, or exploring a genuinely different eviction philosophy — such as adaptive segment-ratio balancing — that does not share the local optimum of the current SLRU design.


## Rejected variants

- v001 — implement
  agent 'variant-implementer' returned status=ok but recorded 0 input AND 0 output tokens (usage=input_tokens=0, output_tokens=0, cache_read_tokens=0, cache_write_tokens=0, cost_usd=0) — almost always a silent backend failure (session limit, quota, dropped connection). tail of /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007/variants/v001/attempts/02/variant-implementer.log.md:
  # Agent log: variant-implementer
  
  - adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
  - workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007/variants/v001/attempts/02
  - allow_edits: True
  - started: 2026-06-04T22:53:18+00:00
  
  ## timeline
  
  ### 22:53:21 chat
  
  Prompt is too long
  

## Write

Exactly one file: `cycle-insight.md` in the output dir.
