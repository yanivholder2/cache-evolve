# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 16384): Moving the reset back from `16 * capacity` to `10 * capacity` halves probation victims' counts sooner. On web_0, where reuse follows recency, a re-referenced candidate with a low estimate now ties or beats a stale victim a bit more often. The gain is small, though. It stays below the recovery level that H01 set as its test, so the reset period explains only part of v002's web_0 loss at small capacity.
- hit-rate (web_0 @ 131072): A shorter halving period lets the sketch drop old popularity faster when web_0's working set shifts. Here the window is large enough that admission only refines the result, so the gain is small but positive.
- hit-rate (hm_0 @ 131072): A marginal gain for the same reason, faster aging. At this capacity probation has spare room, so the extra one-shot admissions that hurt the small size (below) cost almost nothing.

## Hurt

- hit-rate (hm_0 @ 16384): This is the largest move in the variant, and it gives back a real part of v002's gain. Halving happens more often, and each halving also clears the doorkeeper. So warm probation victims fall to low estimates sooner and lose their doorkeeper bit sooner. That leaves them closer to a one-shot candidate's estimate of 1, and the tie-admitting duel (`>=`) lets more scan blocks displace them. At tight capacity, each of those swaps is a lost hit. This shows that v002's longer reset was a large part of its hm_0 one-shot filtering, alongside the doorkeeper and dedup.

## Takeaway

The reset period trades off directly between the two traces at small capacity. A longer frequency memory protects hm_0's warm blocks from scans, and a shorter one helps web_0's recency-driven reuse a little. Here the hm_0 cost is clearly bigger than the web_0 benefit. The reset period is not the main cause of v002's web_0 @ 16384 weakness, so the doorkeeper (which makes first-touch candidates look colder) or the window size is the more likely lever. The `16 * capacity` reset should be kept for hm_0's sake.
