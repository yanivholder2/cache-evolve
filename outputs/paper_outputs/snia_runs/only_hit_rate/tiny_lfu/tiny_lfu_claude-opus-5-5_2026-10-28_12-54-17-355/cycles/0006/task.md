# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0006

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
