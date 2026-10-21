# Variant v002 Insight

## Helped

- hit-rate: Target hits now move into the probing bucket whenever it has room, not only for the hottest item. Spare probing capacity gets used instead of sitting idle, so fewer target-bucket evictions spill to the stash. The gain over the parent is small.

## Hurt

- No dimension measurably regressed. The fitness vector only reports hit-rate, so metadata-size and per-op-time are unmeasured here and can't be compared with the parent. The extra `is_full()` check and the `remove_at` + `promote_to_probing` move on non-hottest hits add work to the `get` path, so per-op-time may be worse.

## Takeaway

Relaxing the hottest-only promotion rule to "promote if the probing bucket has room" gives only a marginal hit-rate gain on this trace. Probing-slot spare capacity is mostly not the binding constraint at this cache size. Promoting non-hot items also lets them displace probing residents, which may offset the gain once buckets fill.
