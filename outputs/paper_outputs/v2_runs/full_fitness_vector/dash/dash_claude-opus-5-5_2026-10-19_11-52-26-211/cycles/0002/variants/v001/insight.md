# Variant v001 Insight

## Helped

- hit-rate: demoting items to the stash's MRU end (`DEMOTE_TO_STASH_POS = Back`) in both `demote_from_normal_bucket` stash inserts lets an item that has already shown reuse survive longer in the stash, so it can be promoted back on its next access instead of being the next stash victim. The result matches the hypothesis's predicted value exactly. That confirms the port composes cleanly with v003's lookup order: reordering the checks changed nothing about which items are kept.
- per-op-time (get p99, put p99): both tails came down slightly compared with the parent, though no get or put code path changed apart from the stash insert position. The likely cause is the different cache state. More hits are served before the fallback to a miss, which scans every tier. Treat this as weak evidence until it is re-measured.

## Hurt

- per-op-time (get p50, the summary dimension): the get code is byte-for-byte v003's target → probing → stash order with lazy `stash_idx`, yet the median rose slightly. Keeping demoted items alive in the stash moves more hits onto the slowest hit path: stash lookup, `promote_to_target`, and possibly a cascade demotion. So part of the extra hit-rate is paid for with more expensive hits. The gap is small enough that it could also be run-to-run noise.
- per-op-time (put p50): rose slightly. Put code is unchanged, so this is either noise or a side effect of a fuller, longer-lived stash.

## Takeaway

Stash insert position and lookup order really are independent. The combined variant gets the higher hit-rate from Back-demotion with no metadata cost, and keeps v003's lookup structure as it was. What did not fully carry over is v003's get-p50 edge. Back-demotion turns some misses into stash hits, and stash hits are the most expensive hits in this design. So the next gain on get latency should come from making the stash-hit → `promote_to_target` path cheaper, not from changing lookup order again.
