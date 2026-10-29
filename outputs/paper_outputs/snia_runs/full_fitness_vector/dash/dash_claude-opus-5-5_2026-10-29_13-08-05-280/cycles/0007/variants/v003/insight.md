# Variant v003 Insight

## Helped

- per-op-time (get p50 and p99): The `PROBE_WINDOW` age check makes v001's two-bucket swap fire less often. A second hottest hit that comes more than one segment's worth of inserts after the first is now treated as a fresh first hit. That path writes only `lfu_counter` and `timestamp` and returns. Before, it did two removes, an insert at `promote_pos`, and a write-back across two buckets. Skipping the swap on those hits takes inline work off the get path, and the tail gains most. This is the partial offset the hypothesis predicted, and it turned out larger than the hit-rate effect.
- per-op-time (put p99): Put code was not edited, yet the put tail improved. A likely cause is that fewer get-side swaps mean less cross-bucket rewriting. That leaves probing and target buckets less churned (and cache-line-hot) when later inserts run their full-bucket path. This is an indirect effect, so it may be partly measurement noise.

## Hurt

- hit-rate: The merge did not add up. Both web_0 cells are unchanged from v001, and both hm_0 cells dipped slightly, the 16384 cell most. hm_0-131072 stayed below the 0.4440 that v002 reached alone, so the hypothesis's own refutation condition is met. In small caches, `PROBE_WINDOW` (the segment slot count) is short in tick terms. Some genuinely recurring keys get their swap delayed or reset, so they stay in the target bucket longer. The large-cache filtering benefit never appeared, because the swap already keeps the demoted probing resident cached. Keeping out slow-recurring keys no longer saves an eviction.

## Takeaway

On top of the direct swap, v002's recency window adds no hit-rate. The swap already fixed the cost the window guarded against: a promotion that evicts a still-useful resident. So the two levers overlap rather than stack, and the window filters swaps that were mostly harmless. Its real value here is as a cheap throttle on the get-path swap. It recovers much of v001's get/put tail latency at almost no hit-rate cost and without growing metadata. Caveat: I couldn't list or open the overlay source in this session, so the code-level reasons assume H03 was implemented as written.
