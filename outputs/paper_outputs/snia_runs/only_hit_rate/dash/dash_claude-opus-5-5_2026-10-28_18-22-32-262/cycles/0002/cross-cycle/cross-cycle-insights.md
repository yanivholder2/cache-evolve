## Recurring wins

- Changing where items are placed in the stash has helped more than adding filters. Putting demoted items at the stash MRU end (Winner in cycle 0001, and the base of cycle 0002) raised hit-rate on every trace and size, and it still holds as the base.
- The two-hit exit gate was the best add-on in both cycles. It won cycle 0002, built on top of MRU demotion. Its gains are steady at the small cache size, where the target tier is crowded and protecting reused items matters most.
- Gains come mostly from cells where reuse comes soon or contention is high: the small size, and `hm_0` at the large size. Levers that protect reused items from one-hit arrivals pay off there.

## Recurring failures

- Every extra stash filter has cost hit-rate on `web_0` at the large size: the fixed gate (both cycles), frequency comparison, and LRU-end insertion. When target buckets have spare room, plain first-hit promotion is already close to optimal. No variant has recovered this cell yet.
- Stash levers interact instead of adding up. They all act on the same tiny bucket-local population, so one can undo another. For example, resetting counters on demotion cancels the benefit of MRU-end demotion.
- Frequency signals are too weak in this design. Counters reset or halve on moves, and promotions land at the target's eviction end, so the TinyLFU-style comparison ended up acting like first-hit promotion.
- Inserting fresh puts at the stash LRU end takes away new keys' probation hit. In effect it means "evict on next collision".
- per-op-time and metadata-size have not been reported in any cycle. Every cost claim (lookup reorder, cheaper gate) is unverified, and cost-only control variants have given no usable signal.

## Open questions

- Does applying the two-hit gate only to fresh puts remove the `web_0` large-size regression? In that version, demoted items keep their count or promote on their first stash hit.
- Would a gate that scales with occupancy or capacity keep the small-size gains while dropping the gate in roomy caches?
- Would frequency admission work if the victim were the bucket's lowest-counter item instead of LRU index 0, and counters survived moves?
- Can a softer fresh-key insertion (one slot above LRU, or a per-bucket bimodal choice) keep the `hm_0` large-size gain without starving new keys?
- Does the target-first lookup reorder lower get latency? Its hit-rate is identical to baseline, so it is risk-free to add, but it is unproven as a cost win until per-op-time is measured.

## Per-policy notes

- Stash/target two-tier design: what goes wrong is the stash ordering, i.e. reused items getting evicted before fresh or never-hit ones. Fixes to entry position and exit criteria have been the most productive levers.
- LRU-within-bucket stash: the slots are tiny, so position changes have large effects. LRU-end placement is almost the same as immediate eviction.
- Frequency/TinyLFU-style admission: it has not worked so far because the counters never build up. Fix counter persistence and victim choice before trying it again.
- Promotion gating: it shows a size-dependent tradeoff (helps the small size, hurts the roomy large size on `web_0`). That makes it a candidate for adaptive or occupancy-aware thresholds.
