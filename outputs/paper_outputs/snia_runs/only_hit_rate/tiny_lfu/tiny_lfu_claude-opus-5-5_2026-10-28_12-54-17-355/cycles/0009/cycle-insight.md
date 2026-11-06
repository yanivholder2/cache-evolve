# Cycle 0009 Insight

## Cohort outcome

Each variant followed one strand of the cycle 0008 continuation. v001 replaced cycle 0008 v003's sticky reuse flag with a "late window hit" signal measured against the candidate's own residency. v002 sent protected-overflow demotions to the probation head, aimed at hm_0. v003 combined single counting with ceiling halving, so the resident wins the rounding. v001 won. It recovered both large cells while giving back part of the web_0 small gain. Its summary moved only slightly, so whether it beats cycle 0005 v003 outright is still unclear. v002 regressed in all four cells. v003 was a near-zero-sum swap.

## Patterns

- Normalising to residency worked. Measuring the reuse signal against how long the candidate has been resident removed the large-cache drift seen in cycle 0008. This is the first tie-related change that did not hurt web_0 large. It confirms that the rule needs recency evidence scaled to residency, not just any recency signal. The fixed quarter cutoff is too strict for short windows, so the signal that pays off lies somewhere between "any hit" and "late hit".
- The probation head is part of the admission contest. The victim is the minimum over the head sample, so whatever is placed at the head sets the bar. Putting proven demotees there raised the bar and also evicted entries with real reuse history. "Placement" and "admission" cannot be changed separately in this design.
- Rounding in the resident's favour fixed the reopened bar, as predicted: web_0 large held. But the aging period scales with capacity, so halving fires often at the small size and almost never at the large size. The effect is decided by how often resets happen, not by how they round. This is another case of the recurring "capacity-scaled horizon" problem.
- hm_0 small moved only under v003, and only by draining web_0 small. hm_0 small and web_0 small now look directly traded against each other through the strength of the frequency contest.
- Per-op-time and metadata-size are still unmeasured, for the ninth cycle running.

## Open questions

- Would a looser or window-scaled late-hit cutoff (a different `LATE_HIT_DIVISOR`) keep v001's large-cell recovery and restore web_0 small?
- In v003, does the web_0 small loss come from halving or from single counting? Would aging on a fixed, capacity-independent period, with double counting kept, separate the two?
- Can lengthening a new admission's probation stay help hm_0 without changing what the head sample sees? Examples: placing contest winners differently, or changing how far the sample reaches.

## Continuation

Use v001 (residency-normalised late-hit tie) as the base, and tune its cutoff fraction or make it scale with window size. This is the most promising thread. Drop "demote to probation head" and any placement change that fills the sampled head with high-count entries. Retry aging only on its own with a period that does not scale with capacity, so it can be separated from single counting. Treat any gain on hm_0 small that costs web_0 small as no progress.
