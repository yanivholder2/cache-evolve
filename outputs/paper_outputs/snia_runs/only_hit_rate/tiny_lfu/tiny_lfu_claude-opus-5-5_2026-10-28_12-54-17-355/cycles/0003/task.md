# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0003

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0003/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
