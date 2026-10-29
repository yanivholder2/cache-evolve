# Variant v002 Insight

## Helped

- hit-rate: Doubling `PROBE_WINDOW` let a few more slow-recurring hot keys in the small hm_0 cache finish the full-probing swap instead of being reset to a fresh first hit. That was enough to lift the hm_0-16384 cell, and with it the summary, by the smallest measurable step. The other three cells did not move, and neither hm_0 confirmation target from H02 was reached, so the gain is real in direction but too small to matter.
- per-op-time (put p99): Put code was not changed. The small improvement in the put tail has no mechanism tied to this change and is best read as noise.

## Hurt

- per-op-time (get p50 and p99): A window twice as wide admits many more second hottest hits into the two-bucket swap path (two removes, an insert at `promote_pos`, and a write-back) instead of the cheap `lfu_counter`/`timestamp` reset. That gives back the throttle v003 gained. The get tail ended up well past the cycle 0006 v001 level that H02 named as its refutation threshold. The median also rose. This suggests the extra swaps are not rare tail events: they now fire on a large share of hottest hits.

## Takeaway

H02's refutation condition is met. Widening the window bought almost no hm_0 hit-rate and cost a large rise in get latency. So the window is not what holds back hm_0 on the swap lineage. It works almost purely as a throttle on how often the costly swap runs, and the parent's slot-count setting is already close to the useful end of that trade-off. Further hit-rate work should change the arm's mechanics or the hm_0-specific eviction behaviour, not the window length. If the window is tuned again, it should be narrowed (or kept the same) for latency, not widened. Caveat: I couldn't list or open the overlay source in this session, so the code-level reasons assume H02 was implemented as written. A rise this large in get p50 is also consistent with an implementation side effect (for example, computing the bound on every get instead of once per segment), and that is worth checking in the overlay.
