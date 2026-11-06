# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002/hypotheses/H03.md

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
