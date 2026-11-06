# Variant v002 Insight

## Helped

- per-op-time: Get p50 and p99 both dropped. On the ClassicLRU path, a hit in the last `n/8 + 1` slots now returns right away. It skips the dynamic `eviction_policy.get_from_position` call and both memmoves (the `items` remove/push and the `tags` remove/push), and the skewed hm_0/web_0 hot keys mostly sit in that window. Put also got faster at p50 and p99. Part of this is plausibly real, because the put-update path calls `get_from_position` and gets the same skip. But the put-miss path (fused rotation, BIP) is unchanged, so part of the put gain is probably the run-to-run variance the parent insight warned about.

## Hurt

- hit-rate: The cells moved by about one ten-thousandth or less, in both directions: the two web_0 cells went down slightly, hm_0@131072 went up slightly, and hm_0@16384 did not change. The summary is far inside the hypothesis's ±0.0005 budget. Leaving a hit in place only changes its rank among the top ~n/8 items, which are far from the index-0 eviction end. Under quarter-depth LIP insertion, that almost never changes which item gets evicted.

## Takeaway

Lazy promotion succeeded where earlier hit-path rewrites failed. It removes the move entirely for near-MRU hits instead of replacing the move primitive or adding swaps, and that clearly lowered get latency at no measurable hit-rate cost. Since `n/8` left hit-rate essentially flat, the hypothesis's own follow-up applies: try the wider shift 2 (`n/4`) to skip more promotions. Watch whether the window starts to overlap the quarter-depth insertion region, because that is where eviction order would begin to change.
