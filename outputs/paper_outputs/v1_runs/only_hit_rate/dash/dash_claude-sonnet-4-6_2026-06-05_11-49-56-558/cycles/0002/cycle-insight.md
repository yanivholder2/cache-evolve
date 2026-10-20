# Cycle 0002 Insight

## Cohort outcome

This cycle probed three directions branching from Cycle 0001's winning topology: compounding MRU re-entry with the 50/50 split, pushing the split further to 75/25, and introducing aging-LFU from a fresh baseline. The sole Winner, v002, confirmed that the normal/stash gain curve had not yet peaked at 16/16 — expanding to 24 normal buckets produced a further lift with no regression, and stash can tolerate reduction to at least 8 slots without penalty.

## Patterns

- **Hot-tier expansion is still the dominant lever.** v002's 75/25 split won outright, reinforcing Cycle 0001's finding that topology is more impactful than policy changes. The working set fits comfortably within 24 normal buckets, meaning capacity was still the binding constraint after the 50/50 rebalance.
- **MRU re-entry and topology are additive but unequal.** v001's compound of MRU re-entry atop the 50/50 geometry improved on v003, confirming the two axes do not interfere, but the marginal gain was modest — once the hot tier is enlarged, collapsing the re-entry penalty matters less because items already have more dwell time.
- **Aggressive per-hit aging is harmful for this workload.** v003's symmetric decay (−1 to all co-resident items on each hit) drove mildly-warm items to eviction before they could contribute hits that recency-ordered LRU would have preserved. Workload-a's access distribution punishes coarse frequency discrimination more than it rewards it.

## Open questions

- Does the normal/stash gain curve continue past 24/8, or does a further push (e.g., 28/4) hit a breakpoint where stash under-capacity starts hurting cold-item re-entry?
- Would stacking MRU re-entry (v001's insertion-position change) onto v002's 75/25 topology produce further additive gains, as it did in Cycle 0001's compound test?
- Is there a gentler aging-LFU formulation — smaller decay increment or access-count-gated decay — that avoids draining mildly-warm items on workload-a?

## Continuation

The topology gain curve is still ascending; the next hypotheses-writer should probe 28/4 and possibly the theoretical limit of maximising normal buckets to locate the peak or breakpoint. Combining MRU re-entry with v002's 75/25 split is the natural low-risk compound to run in parallel. Aging-LFU in its current aggressive form should be dropped; any return to frequency-based eviction should use a finer-grained decay to avoid the mildly-warm eviction problem observed here.
