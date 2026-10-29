# Variant v002 Insight

## Helped

- hit-rate (web_0 @ 16384): This was the cell the hypothesis targeted, and it gained the most by a wide margin. That backs up the parent insight's view that this cell is limited by the window, not by sketch noise. The parent had a fixed window of about 163 entries, too small to hold web_0's recency-driven reuse. With a 50,000-access sample period and a first step of +256, the climber can grow the window to many times that size within a few rounds. `prev_hit_rate` starts at 0.0, so the first move always grows the window, and on this trace growing was the right direction. The grow path also evicts a one-shot main victim (estimate <= 1) even when the candidate loses the duel. That keeps recently re-touched window entries that the parent would have pushed out.
- hit-rate (web_0 @ 131072): A small gain, where the parent cycle's climber lost. The difference is in the caps. At this size the climber gets the same 50,000-access period and 256-entry step as at 16384, not the step of about 8192 that a capacity-scaled design would use. So window moves are small relative to the cache and are re-checked often. Growth also keeps the TinyLFU duel, so frequent probation/protected entries are not flushed while the window grows.
- hit-rate (hm_0 @ 16384): A small gain. Because `min_win` equals the parent's 1% window, the climber can never shrink below the window that served this scan-heavy trace well. The sketch (doorkeeper, dedup, 4x width) is unchanged, so the one-shot filtering from the parent stays in place. The climber probably found a modest window increase that catches hm_0's short-range reuse without letting scans through.
- hit-rate (hm_0 @ 131072): A marginal gain from the same mechanism. With this much capacity, the main area already held hm_0's reuse set, so changes to the window had little left to recover.

## Hurt

None. Every trace/size cell improved on the parent.

## Takeaway

Adaptive window sizing stacks cleanly on v002's tightened sketch, but only when its two scale-dependent knobs are capped: a sample period of at most 50,000 and a step of at most 256. Window growth also has to stay behind the admission duel. The sketch filters out one-shot traffic, and the climber gives web_0 the larger recency window it needed. Because the climber can't go below the 1% window, it has no downside on hm_0. Both sizes tested here hit the same caps, so the gains can't separate the effect of the period cap from the step cap. The next variants should vary them one at a time. They should also log the final `win_cap` for each cell to show how far the window actually moved.
