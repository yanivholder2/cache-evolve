# Cross-Cycle Insights (cycles 0001–0004)

## Recurring wins

- Structural changes to the admission contest keep working. Changing who competes (the window), which resident is the victim (SLRU probation), or how big the window is (the adaptive climber) each won a cycle. Each one also added to the gains of the one before it (1% window → adaptive window → adaptive window + 80/20 SLRU).
- SLRU's probation/protected split is the most reliable component. It helped web_0 in every cycle where it was tried, and it was the only change that moved web_0 at the large size once the climber had stalled there.
- Strict `>` admission, where the victim wins ties, has held up. Relaxing it (recency tie-break, `>=` at a frequency floor) regressed on both traces.
- Gains are consistently largest at the small cache size and shrink at the large size.

## Recurring failures

- Changes to timing or feedback-loop knobs have failed in three cycles running: the aging period, the climber's sample period (tied to capacity, fixed, or tied to turnover), and alternating-turn climbers. In each case the feedback signal was noisy, lagged behind lazy rebalancing, or was biased by integer truncation, warmup, or a first move that always went the wrong way.
- Global periodic sketch halving fixed one problem in isolation (the cap-at-15 regression went away) but hurts once the adaptive window exists. It wears down the main region's frequency lead, and it hurts hm_0 at the small size the most.
- Counter caps combined with strict `>` cause ties at the cap and reject hot newcomers. Dropped.
- Metadata-size and per-op-time have gone unmeasured for four cycles, even though every continuation asked for them. Every winner adds cost (region tags, extra lists, demotions, sampling), and none of it has been quantified.
- Since cycle 0004 the lineage looks close to a local optimum: the latest winner is basically flat against its parent, and the other variants regressed.

## Open questions

- Sketch double count: a miss followed by an insert counts the key twice, so a key seen once already has an estimate of 2. Does fixing this on its own change hit-rate, especially on hm_0? Does it make frequency floors usable? Does it explain the earlier problems with aging periods?
- Can a counterfactual signal (ghost list or shadow cache) replace hill-climbing on noisy deltas, for the window size and for the protected fraction?
- Is the 80/20 protected split already close to optimal, or can a climber with a float step and properly separated knobs learn a better split?
- Aging aimed at specific entries (only demoted protected entries, or gated on the climber's state) instead of global halving.
- What do the current lineage's components actually cost in metadata-size and per-op-time?

## Per-policy notes

- **TinyLFU baseline**: its weakness is the admission contest, which rejects every first-time key once the cache is full, not its eviction order.
- **W-TinyLFU window**: the fixed 1% window was too small at small sizes. The adaptive climber helps at small sizes and is ineffective at large sizes. Every attempt to retune the climber's timing has failed.
- **SLRU main region**: strong on web_0 (scan-like, one-hit traffic). On hm_0 it is flat or slightly worse, because the short probation stay cuts recency-driven entries short and stale demoted entries keep inflated counts.
- **Sketch aging**: helps web_0 when there is no adaptive window. It is harmful on the current lineage and has been dropped there.
- **hm_0**: the hard trace in every cycle, and hardest at the small size. Plain LRU already captures most of its reuse at the large size, and every change has dipped it somewhere.
