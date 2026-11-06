# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0007

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
