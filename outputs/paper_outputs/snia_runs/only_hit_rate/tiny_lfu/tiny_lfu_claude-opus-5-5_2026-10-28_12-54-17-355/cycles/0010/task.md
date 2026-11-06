# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants branched from cycle 0005 v003 (coldest-of-4 sampled victim, double counting kept). Each changed only the sketch: periodic halving every 10×capacity gets (v001), conservative update (v002), and hashing once with double hashing in a flat array, which is a latency-only change (v003). v002 was named winner, but only relative to the cohort. It lifted hm_0 small and lost web_0 at both sizes. No variant improved on the parent, and every variant hurt web_0.

## Patterns

- In this lineage the sketch is not a measurement tool. It is the admission policy itself. Each variant changed the sketch's error profile, and each change moved the admission bar in a way nobody intended. Halving zeroes the cold probation tail, so newcomers clear the bar too easily. Conservative update produces exact low-end ties, which raise the bar under strict `>`. Correlated double-hash collisions tip close contests. The parent's web_0 gain depends on a finely balanced mix of double-count inflation, random collision noise and strict `>`. Any change to one of these pieces upsets the balance.
- Ties and near-ties at low counts decide outcomes, not collision inflation. The sketch is wide enough that inflation of the minimum is rare. Most decisions are made at the low end of the count range.
- Aging on its own does not add recency. It amplifies the double-count newcomer bias. This refutes the cycle 0006 claim that "the unaged sketch is the root cause" in isolation. Aging and the double count must change together.
- hm_0 small moved slightly up in all three variants, which is the first time in several cycles. Stickier incumbents (v002) helped it most. This hints that hm_0 small wants fewer false admissions, not more recency.
- Row independence in the sketch carries real accuracy on web_0 large. Cheaper hashing is not free.
- Per-op-time still cannot be scored, because the fitness vector holds only hit-rate. A pure latency change like v003 can only lose. Seven cycles have now flagged this.

## Open questions

- Does aging help if it is paired with single counting? Removing the double increment is what keeps post-reset zeros from turning into free admissions.
- Does conservative update plus a tie-break in the candidate's favour (`>=`, or `>=` only above a small floor) keep v002's hm_0 small gain and recover web_0? This would revisit the cycle 0004 tie-break verdict now that the sketch has changed.
- Can a per-op-time dimension be added to the evaluator, so that latency work becomes rewardable?

## Continuation

Keep cycle 0005 v003 as the base. No cycle 0007 variant beat it. Stop making one-knob sketch changes. Next cycle, try paired changes that keep the admission bar where it is: aging combined with single counting, and conservative update combined with a tie-break toward the candidate. Treat the sketch, the double count and the `>` rule as one coupled unit. Drop pure-latency hypotheses until per-op-time is measured, and keep independent row hashing.


### Cycle 0008

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


### Cycle 0009

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
