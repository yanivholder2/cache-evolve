# Variant v002 Insight

## Helped

- hit-rate (small-cache cells): Both 16384-size cells moved up a little from the parent, and hm_0 moved slightly more. At this size `settings.capacity()` is below `GUARD_MIN_CAPACITY`, so `full_target_guard` is off. A stash key that reaches two hits enters its target even when the target is full. Before, that key had to wait for a third hit, and in a short-lived stash it was often evicted before getting it. This is the slip v001's insight blamed on the fixed 3-hit gate, and gating by size recovers it. The gain is small and in the expected direction, but the size alone does not show whether it beats noise.
- hit-rate (summary): The summary rose because the small cells gained while the large cells held steady. The size gate removes the trade-off between small and large caches that kept v001's summary flat.

## Hurt

(none: the large-cache hit-rate cells match the parent exactly, so v001's protection of reused target residents is fully kept where the guard stays on)

## Neutral / noise

- metadata-size: No change. The new `full_target_guard: bool` lives in `DashSegment`, and neither measured type (`Item<u64,u64>`, the `Dash` handle) contains it.
- per-op-time: The get median is the same and the other percentiles moved only marginally. In small caches the bool short-circuit skips the `is_full()` load, but that runs only after a second stash hit, so it is too rare to show up. Read these small shifts as run-to-run variance, not a real effect of the change.

## Takeaway

The extra stash hit for full targets is only worth requiring when stash residence is long enough for a key to collect it. Turning the guard on by total capacity (cutoff at the geometric middle of the evaluation sizes) keeps every large-cache gain from v001 and returns the small-cache cells to promoting at two hits, with no cost to metadata or latency. The small-cache recovery is modest, so the 3-hit guard was only a minor drag at 16384. More headroom there will have to come from something other than this threshold. This variant does not show where the cutoff should sit between the two sizes, because only two sizes were evaluated.
