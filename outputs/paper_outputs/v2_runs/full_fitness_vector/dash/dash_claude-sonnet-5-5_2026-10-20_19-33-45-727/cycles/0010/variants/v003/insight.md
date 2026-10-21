# Variant v003 Insight

## Helped

- hit-rate: Preferring victims whose "seen again" bit is clear keeps re-referenced items resident, giving a small gain over baseline's eviction choice. The gain is modest.
- per-op-time: Both get and put latencies fell well below baseline, in the median and the tail. The eviction scan is a single pass over the scanned buckets, and the hit path only sets one bit, so the work per op is small. I could not inspect the overlay source, so I can't say which part of the change drove most of the drop. Some of it may be measurement variance.

## Hurt

- metadata-size: No effect. Packing the flag into spare bits kept the item and table sizes the same as baseline, so this dimension neither gained nor lost.

## Takeaway

A one-bit second-chance flag on eviction, with no admission filter, adds a small hit-rate gain at no metadata cost. Eviction-side second-hit signalling therefore recovers only a little of what an admission-side approach is aimed at. The latency improvement is large relative to the hit-rate gain. It should be checked for noise before it is credited to this change. This note is based on the hypothesis and fitness vectors only, because the source diff was not readable.
