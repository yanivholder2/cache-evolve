# Cycle 0005 Insight

## Cohort outcome

All three variants extended Cycle 0004's capped ghost admission. v001 grew the GCLOCK ghost ring from 4 to 16 entries. v002 moved capped ghost admission onto the 1/8 LRU line, where ghost hits enter at index 3 instead of index 1. v003 added a per-bucket adaptive duel between strict and lenient newcomer grace on GCLOCK. v003 is the listed winner, but only because it tied its parent. It stays on the strongest line in absolute terms, yet H03 was refuted and nothing new was learned about leniency. v002 was the only variant to beat its parent, and only by a small margin. v001 regressed.

## Patterns

- The ghost signal works best when it is narrow. A small, recent ring is selective. Stretching it to cover keys with longer reuse distances diluted the counter-1 bonus, and those keys pushed out more valuable residents. Ring size behaves like the depth axis did in Cycle 0004: the peak is at or below the current setting.
- Capped ghost admission carries over between policy lines. It helped on both CLOCK (Cycle 0004) and shift-based LRU (v002). That supports the rule from earlier cycles that the size of the reward matters more than the signal that triggers it. The LRU gain is smaller, because the slot behind a 1/8 newcomer already catches most keys with short reuse distances.
- Adaptive controllers fail when they learn from their own outcome. In v003's duel, strict mode produced the negative evidence that kept the bucket in strict mode. The lenient branch almost never fired, so H03 was never really tested.
- This cycle produced no hit-rate gain on the CLOCK-plus-ghost line. Its lead now rests on Cycle 0004's result alone. Per-op-time and metadata-size are still not recorded after five cycles, and all three variants added miss-path scans or per-bucket state.

## Open questions

- Is a ring of 2–3 entries as good as 4, or better? This would also lower miss-path and memory cost.
- Would a properly built duel ever choose leniency? That needs set-dueling leader buckets, negatives counted only for newcomers admitted at counter 0, and a seed near the threshold.
- Is v002's small margin above run-to-run noise? Does it, or any CLOCK-line gain, hold on a second trace or cache size?

## Continuation

Shrink the GCLOCK ghost ring (try 2 and 3). Drop larger rings. Retry the adaptive grace duel at most once, with DIP-style leader buckets and an unbiased negative signal. Otherwise conclude that strict filtering plus capped ghost admission is the right shape. Keep ghost-on-LRU (v002) as a secondary line only. The hit-rate axis on workload-a looks close to saturated, so the next cycle should give priority to recording per-op-time and metadata-size and to validating on a second trace or size, ahead of further hit-rate tweaks.
