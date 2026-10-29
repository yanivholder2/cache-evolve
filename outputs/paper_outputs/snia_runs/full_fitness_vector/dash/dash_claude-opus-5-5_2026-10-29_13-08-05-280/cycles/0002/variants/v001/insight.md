# Variant v001 Insight

## Helped

- per-op-time (get median, the summary statistic): Target and probing hits no longer write `lfu_counter`, so they are back to a bare `get_position` + `get_from_position`. Most gets end on those normal-tier hits. Removing that one write accounts for most of v003's median get penalty.
- per-op-time (get tail): A threshold-reaching stash hit now goes straight to `remove_at` + `promote_to_target`. It no longer checks `is_full()` or reads the victim through `eviction_candidate_index()`, and a refused promotion no longer leaves the item in stash for another attempt. Both changes shorten the slow promotion path that sets get p99.
- per-op-time (put median): Puts got slightly cheaper, which fits with the leaner promotion path. The connection is indirect, though, because `put` itself was not edited.

## Hurt

- hit-rate: Hit-rate fell slightly. Both small-cache cells were essentially flat; the loss is in the two large-cache cells, especially hm_0. With no victim comparison, any 2-hit stash key can now push out a full target's LRU resident, even when that resident is reused more. Bigger caches hold a stable reused working set, so they lose the most from this.
- per-op-time (put tail): Put p99 went up. A likely cause is that every promotion into a full target now forces a demotion into stash. That is more stash churn than v003 had, when the victim comparison turned some promotions away, so tail-latency puts hit fuller stash buckets.

## Takeaway

This answers the open question from cycle 0001: the two-hit stash threshold produces almost all of v003's hit-rate gain. The per-hit counter writes on target/probing hits produce most of its median get cost. The victim comparison gave a small hit-rate benefit at large cache sizes (it protected reused target residents) and slowed get p50/p99 a little. Dropping it trades that small large-cache hit-rate gain for a noticeably faster get path. A cheaper victim guard, for example one used only at large sizes or one that reads a single bit instead of a counter, could win back the lost large-cache hits without bringing back the per-hit writes.
