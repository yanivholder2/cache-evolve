# Variant v002 Insight

## Helped

- hit-rate: The sketch and the map really are independent, so moving the increment after the map probe left every admission decision unchanged. Hit rate matches the parent bit for bit, which confirms the hypothesis's claim that the change preserves behaviour.
- metadata-size: The change adds no struct fields and only adds a stateless `prefetch_hash` method, so the `Node`, the sketch heap and the `TinyLfuCache` layout all stay as they were (no change, kept as required).
- per-op-time: The `get` p99 dropped a lot. That fits the claimed mechanism in the rare case it applies: when a counter line has actually fallen out of the near caches, the early prefetch lets that occasional deep miss overlap the SwissTable probe instead of following it.

## Hurt

- per-op-time: The `get` and `put` p50 both rose clearly, which is the opposite of what the hypothesis predicted. In the steady-state benchmark the sketch (`4 × 8·cap` `u16` counters, a few hundred KB at most) stays mostly cache-resident. That leaves little miss latency to hide. Meanwhile every op now pays for `indices(h)` twice (once in `prefetch_hash`, once in `increment_hash`) and issues four extra `prefetcht0` instructions, all on the critical path.
- per-op-time: The `put` p99 also rose. On the eviction path the victim's `prefetch_hash` comes right before `estimate_hash` with no work in between, so it cannot hide any latency. It only adds a second index computation and four more prefetches. In the full-phase hit path, the `found` index also has to stay live across the sketch update.

## Takeaway

Software prefetching the four count-min counter lines only pays off when those lines usually miss and there is enough independent work between the prefetch and the use. At this sketch size the counters are mostly cache-resident, so the extra index arithmetic and prefetch instructions cost more at the median than the overlap with the map probe saves. Only the rare deep-miss tail on `get` gained. Prefetching is the wrong tool for this sketch. The cycle-0008 "scattered counter" cost would be better attacked by making the four counters for a key share one cache line (a blocked/one-line sketch), which removes the misses instead of trying to hide them. Any future prefetch should at least reuse the computed indices and not be issued with zero distance, as the victim prefetch was.
