# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010

## Winners

- v001
- v002

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants probed the IRR-based SLRU baseline from cycle 7 — narrowing the probationary scan window, widening the probationary segment, and combining IRR eviction with ghost admission — and every variant regressed. The winner (v003) advances on the back of the smallest regression while confirming that the ghost-admission framework is not complementary to IRR-based eviction.

## Patterns

- The probationary half-scan is load-bearing under IRR for the same reason it was under frequency: IRR values are not uniformly MAX throughout the probationary list. Cold newcomers cluster near the LRU head, but demoted entries with modest IRR values are distributed across the second quarter too, and the scan's selection quality degrades measurably when that range is dropped.
- The 80/20 protected/probationary split is at or near the minimum required to hold this workload's active set. Widening probationary from 20% to 30% triggers excess demotion churn that IRR timestamp preservation cannot absorb — the hot working set is dense enough that a 70% protected segment is genuinely too small, mirroring the cycle-2 regression on the frequency baseline for the same structural reason.
- Ghost admission does not complement IRR eviction. The IRR signal loses the long-term access history that frequency halving preserved across demotion cycles. A node with two closely-spaced accesses far in the past is indistinguishable from a recently-bursted node under pure IRR, so combining ghost-set placement with IRR victim selection yields a worse eviction signal than either frequency-with-ghost or IRR-without-ghost alone.
- The two parameter axes that cycle 7 left open — scan fraction and segment ratio — are now settled on the IRR baseline. Both tested in the same direction as on the frequency baseline: leave them at the values inherited from the prior winning design.

## Open questions

- Does any hybrid signal that blends IRR with a visit count (or with the referenced bit) recover the long-term warmth information that pure IRR discards, without reintroducing the staleness problem of cumulative frequency counts?
- With scan fraction, segment ratio, ghost admission, and the IRR-vs-frequency signal swap all now evaluated, are there structural eviction philosophies not yet tried — such as adaptive segment balancing driven by live hit-rate feedback — that could break out of the current plateau?
- Is the IRR baseline's strength specific to the LIRS-generated trace, or does it transfer to other workloads? Multi-workload evaluation is now the clearest open question.

## Continuation

Next cycle should keep cycle/0007/v003 as the confirmed baseline (IRR timestamps preserved across demotion, 80/20 split, LRU-quarter scan in protected, LRU-half scan in probationary, no ghost set). The probationary scan fraction and segment ratio are both settled — neither narrowing nor widening is productive. Ghost admission is exhausted as a direction on the IRR baseline. The highest-value next steps are: (1) a hybrid signal that combines IRR with a coarse visit count to recover long-term warmth information, or (2) multi-workload evaluation to determine whether the current gains are broadly transferable before further parameter tuning.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tested three signal-blending hypotheses against the IRR baseline: reintroducing frequency counting as the primary eviction signal with IRR as a tie-breaker (built on the older frequency baseline), adding a coarse visit-count tie-breaker to pure IRR, and replacing raw IRR with a staleness-weighted product. The winner (v001) advances despite regressing from its own parent — confirming that the frequency-counting baseline carries a meaningfully higher absolute score than the IRR baseline, enough to survive the removal of ghost admission.

## Patterns

- Frequency counting as the primary eviction signal outperforms pure IRR in absolute hit-rate terms on this workload, even after ghost admission is stripped out. The cycle-7 and cycle-8 work on the IRR baseline produced real relative gains from a lower starting point, but the IRR baseline never caught the frequency baseline. The search spent two cycles optimizing from a weaker foundation.
- Ghost admission was not an exhausted direction — it was actively contributing to the frequency-baseline design. Removing it regressed v001, but v001 still won because the frequency signal's advantage over pure IRR is large enough to absorb the loss. Ghost admission remains a positive mechanism within the frequency design specifically; the prior conclusion that it was exhausted applied to the ghost-ghost-with-IRR combination and should not have been generalized.
- IRR-based tie-breaking — whether as a secondary tie-breaker over equal-frequency entries (v001) or as a visit-count secondary over equal-IRR entries (v002) — adds no measurable lift. Equal-candidate collisions under the dominant signal are rare enough on this LIRS-structured trace that secondary tie-breakers never activate consequentially. Both mechanisms are inert.
- Staleness-weighted IRR (v003) produces a genuine small gain on the IRR baseline, confirming that silent tight-burst entries are a real class of misclassification under pure IRR. The gain is modest because the frequency of that pattern is low, but the direction is validated.

## Open questions

- Does restoring ghost admission to v001 (frequency primary + IRR tie-breaker + ghost) recover the regression and yield a combined gain, or does the IRR tie-breaker interact badly with ghost-shortcut promotions?
- Is the staleness-weighting improvement (v003) additive with the frequency-primary signal — i.e., does a staleness-weighted frequency score, or a staleness-weighted IRR tie-breaker, improve on v001?
- Given that two cycles of IRR-baseline work peaked below the frequency baseline, is the IRR signal fundamentally weaker on this workload, or did it simply need a different structural context (e.g., combined with ghost admission)?

## Continuation

Next cycle should use v001 as the new baseline (frequency primary, IRR tie-breaker, no ghost, 80/20 split, LRU-quarter and LRU-half scans). The highest-priority thread is restoring ghost admission to v001 to test whether the regression is recoverable — ghost admission was a net-positive mechanism in the frequency design and was only dropped based on a conclusion derived from the IRR baseline where it is genuinely non-complementary. Second priority is applying the staleness-weighting idea (validated in v003) to the frequency-primary signal. Secondary tie-breakers (IRR-on-frequency-ties, visits-on-IRR-ties) are both inert and should not be revisited.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
