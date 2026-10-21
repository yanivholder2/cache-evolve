# Variant v001 Insight

## Helped

- hit-rate: `select_victim` makes the newcomer contest the lowest-estimate entry among the 8 oldest LRU nodes instead of always the head. This let a few more warm newcomers be admitted, and they replaced colder stale entries instead of being rejected because a hot-but-idle key sat at the head. The direction matches the hypothesis, but the gain is marginal, only a handful of hits per ten thousand requests. That suggests the head of the LRU list on workload-a is rarely a hot key with a cold neighbor nearby.

## Hurt

## Takeaway

On workload-a at size 2048, letting frequency choose among the 8 oldest LRU entries works as intended but barely moves hit-rate. Head-only victim choice was not the main thing holding back the admission contest. An 8-node window is a tiny slice of 2048 entries. The oldest entries are mostly uniformly cold, and the sketch never ages, so stale keys that were once hot keep high estimates and still win contests. That makes sketch aging/reset, or a larger or differently placed sample, a more promising next lever than victim sampling alone. The change adds up to 8 extra sketch estimates on every full-cache miss-put, and this evaluation did not measure per-op time. Its latency cost is therefore unknown and should be benchmarked before this marginal gain is accepted as a net win.
