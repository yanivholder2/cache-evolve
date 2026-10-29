# Variant v003 Insight

## Helped

- hit-rate: The hill climber lets the window grow past the fixed 1% on recency-heavy phases. The gain is concentrated at the 16384-entry size on both web_0 and hm_0, as the hypothesis predicted. At that size a 1% window holds only about 160 blocks, too few to keep a recency burst alive until its second touch.
- hit-rate: A second mechanism also helps. After the window grows, `put_inner` admits newcomers to the under-cap window by evicting the main-region victim directly, with no frequency contest. During a recency phase this skips the TinyLFU filter until the window fills. New blocks get a chance to be hit again instead of being rejected because their counts are still low.

## Hurt

- hit-rate (hm_0 @ 131072): This is the one cell that dropped slightly. At this size the 1% window was already enough, and the frequency-protected main region did the work. Window growth shrinks `main_cap`/`protected_cap`. Contest-free fills then evict frequency-proven main entries. The step never settles: it keeps flipping sign with a floor of 1 entry. So the climber keeps paying this cost even though a fixed small window was already near-optimal.
- per-op-time (put): `put` now always calls `rebalance`. After every resize, that call does up to 8 extra unlink/push moves (window→probation and protected→probation) per call until the debt is paid. Shrinks of up to 6.25% of capacity are spread over many puts, so the bounded budget keeps the cost from spiking on one call. Instead it adds a long run of slower puts, which shows up in both p50 and p99.
- per-op-time (get): Every `get` now also updates a hit/miss counter on the struct and runs the `maybe_climb` threshold check. The climb is marked `#[cold]`, so the measured rise is larger than those few instructions explain. One plausible cause is that the larger function stops `get` from being inlined, or that `on_hit` demotions follow a moving `protected_cap`. That is a guess. The benchmark does not isolate the cause.
- metadata-size (TinyLfuCache struct): The cache struct gains the hill-climber state: `max_window_cap`, the three `sample_*` fields, `prev_hit_rate` and `step`. `Node` is unchanged, so the per-entry cost (the summary) is unchanged.

## Takeaway

Adaptive window sizing is a real hit-rate lever for small caches on MSR traces. It does nothing, or slightly hurts, at 131072 entries, where v001's 1% window already sufficed. Part of the gain may come from the contest-free "fill the grown window" path, not from window size alone. A follow-up should separate the two by restoring the contest on that path. It should also cut the per-op overhead: either sample with a cheaper counter (for example, reuse the existing sketch increment path or count only on misses), or let the climber stop once the step reaches its floor, so it stops resizing and its rebalance moves stop with it.
