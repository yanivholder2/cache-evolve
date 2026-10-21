# Variant v002 Insight

## Helped

## Hurt

- hit-rate: The only behavioural change is the periodic halving of every sketch counter (after 64×capacity increments, counting both `get` and `put`). On workload-a this removed long-run frequency history that the strict-`>` admission contest was using. Halving also rounds down, so neighbouring counts merge (2 and 3 both become 1, and 1 becomes 0). That creates more ties between a newcomer and the LRU-head victim, and under strict `>` every tie rejects the newcomer. So halving erased real frequency differences that v003 kept, and it did not admit newly hot keys any faster.

## Takeaway

This isolates the confound left by cycle 0001: aging alone still lowers hit-rate on workload-a, even with no conservative update and a period about 6× longer than in cycle-0001 v002. The loss is much smaller than that earlier combined variant, though, so the shorter period, conservative update, or both caused most of that damage. Workload-a seems to reward full frequency history over freshness. If aging is tried again, use a much longer period or no aging at all. Before blaming aging itself, also test a non-strict (`>=`) or tie-breaking contest, since halving adds ties that the strict `>` contest settles against newcomers.
