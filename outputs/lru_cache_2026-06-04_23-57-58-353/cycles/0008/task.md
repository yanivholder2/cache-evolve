# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008

## Winners

- v003

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Both tested variants abandoned frequency counting entirely — one replacing the full policy with SIEVE's visited-bit second-chance mechanism, the other switching to inter-reference recency (IRR) timestamps within the existing SLRU structure. The IRR-based variant (v003) won with a substantial hit-rate lift, breaking out of the three-cycle neutral-winner plateau.

## Patterns

- Switching the eviction signal from accumulated frequency counts to recency-of-recency (IRR) is the sharpest gain yet recorded. The LIRS-generated trace rewards the cache for asking "how recently were this entry's last two accesses relative to each other" rather than "how many times has this entry been accessed in total." Frequency counting was the right direction early in the series but accumulated access counts conflate historical warmth with current warmth in ways IRR does not.
- Both survivors share a common insight: a single reuse since insertion provides a more useful eviction signal than total access count. SIEVE acts on "was it touched at all since insertion"; IRR acts on "how tightly clustered were the last two touches." IRR is the stricter, more informative form of the same intuition, which explains why it dominated.
- Preserving IRR timestamps across protected-to-probationary demotion replaces the halving-on-demotion mechanism naturally — a hot entry (small IRR) retains its competitive advantage in probationary without any manual counter adjustment, while cold newcomers arrive with IRR = MAX and lose the eviction contest without special handling.
- The ghost set was dropped in v003 and it won, confirming what cycles 4 through 6 suggested: ghost admission adds no value on this workload and can be omitted from the baseline going forward.
- The adaptive segment-ratio hypothesis (H01) was never evaluated due to an implementer failure. It remains untested on both the old frequency baseline and the new IRR baseline.

## Open questions

- Does IRR-based eviction transfer to non-LIRS workloads, or is it specifically aligned to this trace's generation structure? A single workload cannot distinguish broadly strong policy from a perfectly matched one.
- Is the 80/20 protected/probationary split still appropriate for IRR-based eviction, or does the sharper eviction signal change what the optimal ratio looks like? H01's adaptive balancing idea was never tested and is now more interesting on the new baseline.
- Are the current scan-window fractions (LRU quarter of protected, LRU half of probationary) still well-calibrated with IRR as the signal? IRR is a sharper discriminator than frequency counts, so narrower scan windows might suffice.

## Continuation

Next cycle should carry v003 as the new baseline: SLRU 80/20, IRR timestamps (preserved across demotion), no ghost set, LRU-quarter scan in protected, LRU-half scan in probationary. The highest-priority untested thread is adaptive segment-ratio balancing (H01 transposed onto the IRR baseline) — it was blocked this cycle and directly addresses whether the fixed split is optimal. Second priority is scan-fraction calibration: with a sharper eviction signal, narrower windows may yield equivalent quality with less scan overhead. Multi-workload evaluation is overdue and should be treated as a near-term requirement before further parameter tuning.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
