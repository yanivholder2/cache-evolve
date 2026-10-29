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
