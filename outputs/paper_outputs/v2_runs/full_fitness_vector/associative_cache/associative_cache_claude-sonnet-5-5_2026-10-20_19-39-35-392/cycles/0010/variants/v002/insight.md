# Variant v002 Insight

## Helped

- metadata-size: Unchanged. The three-way rotation only moves existing entries and adds no per-bucket state, as the hypothesis required.

## Hurt

- hit-rate: Slightly lower than the parent. Demoting the len/4 occupant to len/2 instead of the tail did not recover hit rate. The len/2 entry is not clearly colder than the one it replaces, and the cascade pushes that entry to the eviction slot, so the cost of demotion moved rather than disappeared.
- per-op-time: Both get and put got slower. The two extra element moves and the len/2 clamping and fallback branches add work to every put miss. The get median also rose, even though get logic was untouched, which suggests code layout or measurement noise alongside the heavier insertion path.

## Takeaway

Cascading the displaced entry through len/2 does not fix the hit-rate loss from swap insertion. It gives up part of swap's cheap-put advantage (extra moves and branches) without making the eviction choice better. Where the displaced entry lands matters less than the fact that insertion at len/4 pushes a mid-recency entry toward eviction. A fix would need a better signal for which entry to evict, not a longer chain of moves.
