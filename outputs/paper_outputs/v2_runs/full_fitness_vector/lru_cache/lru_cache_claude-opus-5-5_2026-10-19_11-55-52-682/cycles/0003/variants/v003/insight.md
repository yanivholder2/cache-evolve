# Variant v003 Insight

## Helped

(none — no dimension moved in the intended direction beyond noise)

## Hurt

- hit-rate: The forced eviction at the 32nd examined node sometimes removes a key whose `visited` bit was set, so a key with real reuse is dropped where the parent would have cleared its bit and kept it. The loss stays within the hypothesis's budget, but it is a real cost. It also shows that runs of more than 32 visited nodes do occur on workload-a at size 2048.
- per-op-time: The cap did not lower `put` p99, the target of the hypothesis; it stayed flat or got marginally worse. `get` has no code changes, so its small shifts are noise. Bounding `find_victim()` therefore removed little of the work that sets the `put` tail. That tail is more likely set by the work every miss does after the victim is chosen: `map.remove` plus `map.insert` (hashing and probing), the key clone, and cache misses on the touched nodes. Sweeps longer than 32 steps seem too rare to reach the 99th percentile.

## Takeaway

Capping SIEVE's hand at 32 steps traded a small amount of hit-rate for no measurable gain in `put` tail latency. Long visited runs exist often enough to cost some evictions of reused keys, but too rarely to set p99. The parent insight blamed the sweep for its `put` p99 being about 2× its p50, and this variant does not support that. The next attempt at `put` p99 should look at the per-miss HashMap remove/insert path, not the sweep length.
