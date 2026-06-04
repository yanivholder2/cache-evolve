# Variant v003 Insight

## Helped

*(no dimensions helped)*

## Hurt

- hit-rate: filtering `lfu_counter == 0` items out of the stash removed items that would have been promoted on their next access, converting second-chance stash hits into full misses.

## Takeaway

The filter's premise — that `lfu_counter == 0` at eviction time identifies worthless items — does not hold on a Zipfian workload. A moderately popular key inserted during a cold burst can be evicted before receiving any access in that slot, yet still be accessed imminently; dropping it from the stash eliminates the second-chance opportunity entirely. The stash's value comes precisely from holding these just-evicted candidates until the workload circles back to them, and the zero-frequency heuristic discards exactly those candidates, shrinking effective stash utility more than it saves slot capacity.
