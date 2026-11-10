# Variant v002 Insight

## Helped

- hit-rate (hm_0 @ 16384): A small gain, much smaller than what the parent's age gate bought. Some of hm_0's squeezed-out real reuses do come back between one and two small-queue turnovers after being ghosted, and the wider window now sends them straight to main. But most of hm_0's benefit was already captured inside one turnover.

## Hurt

- hit-rate (web_0 @ 16384): This is the outcome the hypothesis said would locate the boundary. On web_0, freq-0 returnees aged between one and two turnovers are mostly late one-off rereferences, not real reuses. Admitting them directly into main costs a main eviction for each one and pushes out entries that are actually reused, instead of letting them take the cheaper path through small, where they would be filtered.
- hit-rate (web_0 @ 131072): A negligible dip. As in the parent, routing freq-0 returnees to main or to small barely changes which key gets evicted at the large size. Doubling the window still did not unstick this cell.

## Takeaway

Widening the ghost-age window from 1× to 2× `small_size` mostly let in returnees that are not real reuses. The cost on web_0's small size outweighs hm_0's marginal gain. So the useful recency boundary is about one small-queue turnover: a freq-0 key that returns within one turnover is likely a real reuse, but between one and two turnovers it is mostly noise, at least on web_0. The large size did not respond to the window in either direction. This suggests web_0 @ 131072 is limited by something other than how freq-0 ghost returnees are routed. The window should stay at 1× (or be probed narrower), not be widened further.
