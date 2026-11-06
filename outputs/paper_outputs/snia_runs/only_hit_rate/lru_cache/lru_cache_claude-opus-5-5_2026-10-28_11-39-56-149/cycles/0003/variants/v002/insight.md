# Variant v002 Insight

## Helped

- hit-rate (hm_0 @ 16384): This is the biggest gain in the variant, and it came from the cell the hypothesis did not target. hm_0 has a lot of one-shot traffic. With the doorkeeper, and with the duplicate increment removed so a get-miss followed by a put counts once, a one-shot block now estimates 1 instead of 2. It also no longer writes into the 4-bit counters. One-shot candidates now lose the admission duel against probation victims much more often. The wider sketch has fewer collisions, so those victims' counts are less inflated by noise and more likely to reflect real reuse. When capacity is tight, every probation slot saved from a one-shot block is a direct hit gain.
- hit-rate (hm_0 @ 131072): The same mechanism helps here, but by less. At this capacity, probation has enough room that letting in a few one-shot blocks cost less in v002 to begin with.
- hit-rate (web_0 @ 131072): A small gain, which fits the cleaner estimates. Here the large 1% window still catches web_0's short-range reuse before admission runs, so sharper frequency estimates only refine an admission decision that was already mostly correct.

## Hurt

- hit-rate (web_0 @ 16384): The hypothesis predicted this cell would improve the most. Instead it slipped further below v002. So the parent insight's guess that this weakness came from sketch noise or double counting is not supported. The likely cause is the reset period moving from `10 * capacity` to `16 * capacity`. Probation victims now keep their old frequency for longer. On web_0, where reuse is driven by recency and the ~160-entry window is too small to absorb it, a newly re-referenced candidate (estimate around 2) loses to a victim whose old count has not yet been halved. The doorkeeper also makes first-touch candidates look colder than before. That is good for hm_0, but on web_0 it pushes out second-access blocks that v002 would have admitted when they tied the victim. This variant changed width, reset period, doorkeeper and dedup together, so it cannot show which one caused the drop.

## Takeaway

Tightening the sketch (dedup on miss-then-put, 4x width, doorkeeper, longer halving period) works as a one-shot filter. It pays off most on scan-heavy hm_0 at small capacity. It does not fix web_0 at small capacity, and makes it slightly worse, which suggests that cell is limited by the window or by recency, not by sketch noise. Longer frequency memory works against web_0's recency-driven reuse. So H01's window lever is the better candidate for web_0 at small capacity, and the reset period should be tested separately from the doorkeeper and width changes.
