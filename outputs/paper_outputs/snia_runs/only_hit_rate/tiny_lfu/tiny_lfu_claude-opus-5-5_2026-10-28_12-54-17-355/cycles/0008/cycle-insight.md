# Cycle 0008 Insight

## Cohort outcome

This cycle tested the paired changes that cycle 0007 asked for. v001 combined halving with single counting. v002 combined conservative update with a candidate-favouring tie-break above estimate 3. v003, a reuse-gated tie-break, came from a separate line of work: it admits a candidate on a tie only if that candidate was hit while in the window. v003 won, but its insight says the small-size gains and large-size losses roughly cancel. So, as in the last two cycles, the win is relative to the cohort and not clear progress over cycle 0005 v003. Neither pairing kept the admission bar level, and both lost web_0.

## Patterns

- Every variant targeted the tie or near-tie at the low end of the counts, and every one failed the same way. When equal frequency is resolved in the candidate's favour, it displaces probation residents that were worth more. This failure now repeats across cycles 0004, 0007 and 0008 under three different sketches. Equal frequency is not a good enough reason to evict a main resident, even with a floor or a reuse gate.
- Pairing did not fix the asymmetry. It moved it. Integer halving rounds a resident's count of 1 down to 0, but the newcomer keeps its 1 until the next reset, so post-reset free admissions came back one count lower. Any aging scheme has to round in the resident's favour, for example by treating a count of 0 as 1 in the contest, or it reopens the bar.
- Signals that scale with window or capacity drift. v003's sticky reuse flag was precise when the window was short and acted like a blanket `>=` when the window was large. This adds to the earlier evidence about capacity-scaled ghosts and halving periods: any recency evidence has to be normalised to how long an entry has been resident.
- Gains on small sizes and losses on large sizes is now the cohort's typical shape. Every change that lowers the bar helps one or both small cells and hurts the large cells, most of all web_0 large.
- hm_0 small moved up slightly in all three variants again, but none of the changes explains its deficit. Two insights again point to where entries are placed in probation, not to how admission is counted or resolved.
- Per-op-time and metadata-size are still unmeasured, for the eighth cycle running.

## Open questions

- Does a reuse signal normalised to window size, such as a hit near the window LRU or hits relative to residency time, keep v003's small-cache gains without the large-cache drift?
- If the estimate-2 tie is tested on its own under conservative update, does it explain the web_0 loss? Or is the loss caused by the way conservative update reshapes the counts more broadly?
- hm_0 has been flat through eight cycles of admission tweaks. Would changing where admissions land in probation, or how long they stay there, finally move it?

## Continuation

Stop changing the tie rule. Global, floored and reuse-gated tie-breaks have all hurt the large cells, so drop them. Keep cycle 0005 v003 as the base unless v003's reuse signal is redesigned to scale with window size, which is the only tie-related idea still worth one try. Aim hm_0 hypotheses at how admissions are placed in probation and how long they stay, not at the admission contest. Retry aging only with rounding that favours the resident or a floored contest.
