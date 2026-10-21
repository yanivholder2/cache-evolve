# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Each `get` hit now adds `HIT_WEIGHT = 2` to the sketch, while a miss on a non-resident key adds only 1. The same request rate therefore builds a resident's estimate twice as fast as an outsider's. Admission is a strict `>` contest between the newcomer and the coldest of the 8 oldest probation entries, so returning keys find it harder to get in, and the cache settles on incumbents. This cost hit-rate slightly instead of improving it.

## Takeaway

Weighting hits does not add new information about demand. It ties a key's frequency estimate to whether the key happened to be resident when it was accessed. That builds a feedback loop: residents score higher because they are resident, and keys that were rejected fall behind even when they are requested just as often. The comparison TinyLfu admission depends on, newcomer against victim in one shared currency, gets skewed toward incumbents. The change was implemented as specified: lookup comes before the increment, puts and update-puts stay at weight 1, and the promotion gate reads the weighted count. The small loss therefore supports the hypothesis's fallback reading. On workload-a, the plain access count is already the right currency, and residents are already protected well enough. The remaining gains are not on the incumbent side of the admission contest, so tilting it further toward incumbents only hurts.
