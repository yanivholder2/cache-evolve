# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0009

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle 0005's sampled probation victim (coldest of the 4 oldest) and followed its continuation. They tried single-count sketch accounting stacked on top (v001), widening the victim sample from 4 to 8 (v002), and a +1 ghost re-entry bonus for recently rejected candidates (v003). v002 was named winner, but its own insight puts its summary *below* the parent: small-size gains came with a loss on web_0 at the large size. So the win is relative to the cohort, not progress over v003. No variant reached any of its stated targets.

## Patterns

- web_0 at the large size was the failure point for every variant. Each change made the admission bar easier to clear or removed a newcomer advantage there. The large-size frequency filter is the lineage's most valuable and most fragile asset.
- The mechanisms overlap rather than add. Single counting, a wider minimum, and the sampled victim all push admission toward "favour accumulated frequency, evict whatever looks coldest". Stacking them gives diminishing or negative returns. The coldest-of-4 sample looks close to the sweet spot. A minimum over more noisy count-min estimates picks residents that only *look* cold.
- The unaged sketch is the common root cause. v001 showed that the "inflated" estimate of 2 on miss-then-insert was the only recency weight newcomers had. v003 showed that, without decay, ties happen almost only among cold keys, so a tie-only bonus just reshuffles cold entries. Two variants independently concluded that the lineage needs real aging or decay before recency-side tweaks can work. This reverses the cycle 0003 verdict on halving, which was made before SLRU and victim sampling existed.
- Any horizon that scales with capacity is risky at the large size. A ghost of capacity/2 let scan or loop keys on web_0 return while still remembered. This repeats the earlier lesson about controllers whose timing scales with capacity.
- hm_0 small dropped again, in two of the three variants. Six cycles of hypotheses aimed at it have not moved it.
- Metadata-size and per-op-time are still unmeasured, now six cycles in a row. v002 doubled the sketch estimates per insert and v003 added a hash set, both for no hit-rate gain.

## Open questions

- Now that SLRU and victim sampling are in place, does sketch aging (halving, or a doorkeeper reset) give newcomers back a recency weight? Does it then make single counting and re-entry bonuses work?
- Would a contest margin or a less noisy victim estimate (for example, the median of the sample, or a conservative-update sketch) sharpen admission better than a wider minimum?
- Does a short, fixed ghost horizon with a bonus that matters at non-trivial counts help hm_0 without leaking scans on web_0 at the large size?

## Continuation

Treat cycle 0005 v003 (4-node sample, double counting kept) as the real best base. v002's 8-node sample did not beat it. Stop sweeping the sample size and stop stacking single counting without decay. The next cycle should test sketch aging or decay on this base as its own change, then retry single counting or a re-entry bonus on top of it only if aging holds web_0 at the large size. Any ghost or bonus must use a fixed, short horizon. Measure metadata-size and per-op-time.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
