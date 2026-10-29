# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Each of the three variants branched from baseline and changed a different part of TinyLFU: an admission window in front of the main cache (v001), segmenting the main cache into probation and protected (v002), and periodic aging of the sketch (v003). All three improved hit-rate. The winner was v001's 1% LRU window, which fixes the baseline's habit of rejecting every first-time key once the cache is full.

## Patterns

- The baseline's weak point is its admission contest, not its eviction order. Each winning change altered one input to that contest: who competes (v001's window), which resident is the victim (v002's probation victim), or how fresh the frequency evidence is (v003's aging).
- The three changes are independent of each other, and together they make up the standard W-TinyLFU design. None of them has been combined with another yet.
- Gains were largest at the small cache size and shrank at the large size, especially on hm_0, where a plain LRU already holds most of the reuse set. web_0, with its scan-like one-hit traffic, gained most from v002's SLRU.
- The only regression was v003 on web_0 at the large size. v003 also capped counters at 15, so many hot keys tie at the cap, and the strict `>` admission rule then rejects tied newcomers. The cap is the likely cause, not the aging.
- Every evaluation measured hit-rate only. The cost of the region tags, the second list, and the sketch resets on metadata-size and per-op-time is unknown for all three variants.

## Open questions

- Do the window, SLRU, and aging gains add up when combined, or do they overlap? For example, a window may already recover much of what aging gains at small sizes.
- Does aging without the counter cap (or with a higher cap) remove the large-size regression on web_0? Would a sketch period not tied to capacity help at large sizes?
- Is a 1% window the best size, given that v001's gain shrinks as capacity grows?
- What do these changes actually cost in metadata-size and per-op-time?

## Continuation

Build on v001. Add v002's probation/protected split to its main region, and add v003's sketch aging without the 15 cap or with a higher one. Test these as separate steps so it is clear what each contributes. Try window size as a separate axis. Make sure metadata-size and per-op-time get measured before the next winner is called a free win. Drop the counter cap combined with strict-greater admission.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants branched from cycle 0001's 1%-window winner, and each added one W-TinyLFU component on its own: an SLRU main region (v001), cap-free sketch halving (v002), and a hill-climbing adaptive window (v003). Each one raised the hit-rate summary. The winner is v003's adaptive window. Its gains came mainly at the small size on both traces, which shows the fixed 1% window was too small there.

## Patterns

- The three changes help in different places, so they look more complementary than overlapping. The adaptive window helps most at the small size on both traces. SLRU helps web_0 at both sizes. Aging helps web_0 at both sizes and hm_0 at the large size. On web_0 at the large size, only SLRU and aging improved anything, so the gap left there is not a window-size problem.
- hm_0 benefits little from these changes or is hurt by them. SLRU left it flat. Aging hurt it at the small size, because the reset period counts sketch increments, and a miss produces two increments. Its steady frequent keys get halved before they can build a lead, and halving creates more ties, which the strict `>` rule resolves in favour of the victim.
- Two of the variants depend on a period tied to `capacity`: aging's reset interval and the climber's sample period. Both behave badly at one end of the size range. Aging fires too early at the small size, and the climber moves too slowly at the large size. How these periods are set is now a cross-cutting tuning axis.
- Cycle 0001 suspected the regression came from the 15-cap, not from aging. v002 supports that: aging with no cap no longer regresses on web_0 at the large size.
- Metadata-size and per-op-time are still unmeasured for every variant, even though the previous cycle asked for them. Each variant adds some potential cost: SLRU adds a third list and demotions, aging sweeps the whole table on each reset (latency spikes), and the climber adds per-get sampling.

## Open questions

- Do SLRU and aging add to the adaptive window when combined? Since they help mainly web_0 at the large size, where the climber did nothing, they seem likely to.
- Can a sample period that does not depend on capacity, or is shorter, make the climber useful at the large size?
- Would counting aging periods per request instead of per increment, or using a longer period for small caches, remove the hm_0 small-size regression?
- Should ties in the admission contest be broken differently (for example `>=` or a random tie-break), now that halving makes ties common?

## Continuation

Build on v003, the adaptive window. Next, layer cycle 0002's v001 SLRU split into its main region. Then, as a separate step, add cap-free aging with a per-request period, or a period scaled up for small caches. Try decoupling the climber's sample period from capacity as its own axis. Measure metadata-size and per-op-time before calling any winner, especially the cost of a full-table sketch reset. Drop the fixed 1% window, and drop aging periods counted per sketch increment.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants branched from cycle 0002's adaptive-window winner, and each followed one part of that cycle's continuation: an SLRU main region (v001), cap-free halving counted per `get` (v002), and a climber sample period that does not depend on capacity (v003). Only v001 won. It finally moved web_0 at the large size, where the climber had been stuck. Aging and the faster climber both regressed overall.

## Patterns

- A structural change worked and two timing changes failed. Changing which resident gets evicted (SLRU) added to the adaptive window. Changing *when* things happen (how often the sketch is aged, how often the climber samples) either interfered with the window or did nothing.
- The prior cycle predicted that aging would add to the window. That was wrong. Once the climber gives newcomers room, halving mostly erodes the main region's frequency lead. It hurt at the small size, where it fires often, and had no effect at the large size, where it fires rarely. Fixing the twice-per-miss period removed the suspected cause but not the regression, so the regression comes from the interaction itself.
- The climber's feedback period has to match how fast the cache responds. Because rebalancing is lazy, moving one entry per insert, a window retarget takes about a full turnover to take effect. A fixed short sample scores window sizes that were never actually reached, and the lagging EWMA comparison plus the step floor let the window oscillate. A sample period tied to capacity was too slow at the large size, and a fixed one is too fast at the small size.
- hm_0 stays the hard trace. All three variants dipped it somewhere. Even the winner lost a little at both sizes, because the small probation segment cuts short the stay of recency-driven entries, and demoted stale entries keep inflated counts in a sketch that is never aged.
- Metadata-size and per-op-time are still unmeasured, for the third cycle running. v001 adds a third list and demotion traffic.

## Open questions

- Should the protected fraction adapt, for example climbed like the window or set from probation hit feedback, so that hm_0 recovers without giving back web_0's gain?
- Could aging that targets only demoted protected entries, or aging gated on the climber's state, fix the stale-count problem without the broad erosion seen in v002?
- Would a climber sample period tied to observed rebalance progress (window length reaching its target) or to cache turnover fix both ends of the size range? Should the window also rebalance eagerly instead of lazily?
- Would a different tie-break in the admission contest matter more now that SLRU decides the victims?

## Continuation

Build on v001, the adaptive window plus 80/20 SLRU. The next axis is the protected/probation split, either made adaptive or swept, judged mainly by recovering hm_0. Drop global periodic sketch halving on this lineage, and drop fixed constant climber sample periods. If the climber is revisited, tie its sample period to rebalance completion or turnover, not to a constant. Measure metadata-size and per-op-time before declaring the next winner.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants branched from cycle 0003's adaptive window + 80/20 SLRU. Each changed one thing: an adaptive protected split using a second climber on alternating turns (v001), admitting the candidate on ties at estimate ≥ 2 (v002), and eager rebalancing with a sample period tied to window turnover (v003). Only v001 was named winner, and its summary is about flat. It gained on hm_0 at the large size and on web_0 at the small size, lost on hm_0 at the small size, and slipped just below the parent on web_0 at the large size. v002 and v003 regressed across the board.

## Patterns

- No variant clearly beat its parent this cycle. The current lineage looks close to a local optimum, and the small changes tried here mostly show up as noise or harm.
- Feedback-loop design was the main failure again. v001 and v003 both broke on how the climber gets its signal. v001's step shrank to 1-point moves through integer truncation, its first move was always in the wrong direction, and alternating turns did not separate the two knobs' effects. v003 paired short, noisy samples with steps scaled to capacity, and warmup rewarded whichever direction it moved first. Over two cycles, every change to climber timing has failed.
- The admission tie-break question is now settled for this lineage. Keeping strict `>`, where the victim wins ties, is better on both traces. Recency is the wrong tiebreaker once SLRU chooses probation victims.
- A cross-cutting defect showed up: one miss followed by an insert counts twice in the sketch. A key seen once therefore already has an estimate of 2, which defeats any admission threshold based on frequency. This may also explain earlier aging-period issues.
- hm_0 at the small size is still the hard point. It regressed in all three variants.
- Metadata-size and per-op-time are still unmeasured, now four cycles in a row. v003 also added a free list and bulk moves during rebalancing.

## Open questions

- Would a protected-split climber with a float step accumulator, a starting direction seeded per workload, or a ghost/shadow estimate learn a real split, or is the 80/20 split already near optimal?
- Does fixing the double count in the sketch (one access counted once) by itself change hit-rate, especially on hm_0, and does it make frequency floors work?
- Can a ghost list or shadow cache give the climber a counterfactual signal and replace hill-climbing on noisy deltas?

## Continuation

Keep building on the lineage, but treat the v001 win as marginal. Next, try fixing the sketch so each access counts once, as a standalone correctness change. If the protected-split climber comes back, use a float or floored step and a signal that truly separates the two knobs, not alternating turns. Drop recency tie-breaking on admission. Drop window-turnover sample periods unless steps scale with the window and the climber is suppressed until the cache is full. Measure metadata-size and per-op-time.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants branched from cycle 0004's adaptive window + adaptive protected split. They tried a sketch fix that counts each miss-then-insert once (v001), an ARC-style ghost-list controller in place of the window climber (v002), and taking the coldest of the 4 oldest probation entries as the eviction victim (v003). v003 won. It is the first clear, regression-free gain in several cycles, and it gained on web_0 at both sizes. v001 traded web_0 small against hm_0 small, and v002 regressed in every cell.

## Patterns

- Every hypothesis aimed at hm_0, and none moved it in the way it predicted. Each one's stated mechanism was refuted. The gains that did land came on web_0 and through frequency-based selection: a sharper admission contest in v003 and less collision noise in the small sketch in v001. Changes that make the cache better at picking by frequency keep paying off. Attempts to rescue recency-driven entries keep missing.
- The admission/eviction structure is still where progress comes from, and feedback-loop redesign is still where it fails. v002 makes five cycles in a row of failed window-controller changes. Ghost lists fit W-TinyLFU poorly: the frequency filter, not the window size, produces most window rejections, so the two ghost lists fill very unevenly and the controller ratchets the window down to its minimum. Applying the controller on every event also sent traffic through the eviction paths that skip the admission contest.
- Position matters more than choice for hm_0. Fresh admissions land at the MRU end of probation, but victim sampling only looks at the LRU end. v001 also showed that hm_0 small had been relying on an accidental bias: the double count gave a frequency bonus to keys that returned after eviction. Taken together, hm_0 seems to want an *explicit* recency or re-entry signal, not a different choice among the oldest few entries.
- Single-count sketch accounting is the correct base, even though it lost on hm_0 small.
- Metadata-size and per-op-time are still unmeasured, now five cycles in a row. v003 adds a few sketch estimates per full-cache insert. v002's ghost sets would have added a lot of metadata.

## Open questions

- Does v003's victim sampling combine with v001's single-count fix? The two gains come through different routes (contest bar and collision noise).
- Does a larger victim sample keep strengthening web_0, and at what point do the extra per-op estimates cost too much?
- Can hm_0 be helped on purpose by giving a doorkeeper/ghost bonus to recently evicted keys when they come back, or by changing where new admissions land in probation or how long they stay there?
- Could a ghost signal work with balanced inputs (only size-caused window evictions, normalised per list) applied in batches at period boundaries with small steps? Or should controller work stop entirely?

## Continuation

Build on v003, the sampled probation victim. First, layer v001's single-count sketch fix onto it, and sweep the victim sample size as a separate step. For hm_0, try an explicit re-entry/recency bonus or a change to where admissions land in probation. Don't try any more victim-choice tweaks at the LRU end. Drop per-event ghost-list window controllers, and freeze the window/protected climbers unless a balanced, batched signal is proposed. Measure metadata-size and per-op-time.


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


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

All three variants branched from cycle 0009 v001, the residency-normalised late-hit tie. v001 loosened the late-hit cutoff from a quarter to a half. v002 added aging on a fixed period with ceiling halving and kept double counting. v003 removed the protected-split climber so the window climber acted on every sample. v002 won, but only relative to the cohort. It gained marginally on web_0 small and lost both large cells, hm_0 large most of all. Every variant regressed at least one large cell, so none clearly beats cycle 0009 v001.

## Patterns

- Each change relaxed a guard on the parent's behaviour: a looser tie gate, decayed resident counts, or one less adaptive knob. Each relaxation cost the large cells. The parent's large-cell recovery depends on keeping both ties and count gaps hard for newcomers to win.
- The late-hit cutoff was not too strict. Loosening it hurt web_0 at both sizes, so the parent's web_0 small gap needs a lever other than the tie rule. Every global, floored, reuse-gated or loosened tie change has now lost web_0. If the tie gate moves at all, it should tighten.
- A fixed aging period flipped the capacity-scaling problem without solving it. The period is far longer than the reuse span at the small size, where it is harmless and slightly helpful. At the large size it falls inside the reuse span, and warm residents lose their frequency lead to newcomers that still receive an undiminished double increment. Aging and double counting work against each other whatever the period or rounding.
- The adaptive protected split carries real weight on hm_0 large. That makes it the first structural knob shown to matter for hm_0. Giving the window climber more turns made it worse, not faster, because its step decays per climb.
- Per-op-time and metadata-size are still unmeasured, for the tenth cycle running.

## Open questions

- Does tightening the late-hit gate, or removing the tie exception entirely at large window sizes, strengthen the large cells further?
- Is aging viable at all while double counting stays? Possible tests: aging only with single counting on a fixed period, or a period at a large multiple of capacity.
- Would retuning the window climber's step decay or sample period, while keeping the protected split adaptive, help the large cells?
- Is there an hm_0 lever on the protected side, such as the protected climber's step floor or bounds, since that boundary has now been shown to do work?

## Continuation

Keep cycle 0009 v001 as the base. No cycle 0010 variant clearly beat it, and v002's large-cell losses rule it out as the new base. Drop looser late-hit cutoffs and any aging that keeps double counting. Keep the protected split adaptive. Climber work should retune step decay or sample period directly, not reassign turns. Worth probing next: a tighter tie gate, and the protected-split climber's step and bounds as a lever for hm_0 large.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
