# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Splicing a ghost returner in at the SIEVE hand gives it almost no time to earn its `visited` bit. It survives only until the next main eviction, while a tail-admitted key survives a full lap of main. On workload-a, ghost returners evidently reuse at distances longer than that short window, so many are evicted before their next access. Main evictions are not recorded in the ghost, so an evicted returner also loses its fast path. On its next miss it re-enters through the small queue and pays probation again. The result is a small loss against the parent, below the refutation bar the hypothesis set itself.
- per-op-time: The splice is O(1), as expected, but both `put` and `get` got slower. `get` is byte-for-byte unchanged from the parent, so its slowdown cannot come from this change. Most of the `put` regression probably shares that cause (run-to-run or environment drift) rather than coming from `insert_at_hand_main`. Making returners the likely next victim should, if anything, shorten hand walks. This dimension needs a re-run before any of the movement is credited to the policy.

## Takeaway

On workload-a, the cost of ghost recall is not that returners hold main slots too long. Taking protection away from returners loses more hits than it frees up for proven-reuse keys. A ghost hit is a better reuse signal than one hand-gap of probation can confirm. That is especially true because a returner evicted from main gets no second ghost entry. So the "softer admission" direction behaves like a weaker ghost, not a more selective one. Future work on the ghost should look elsewhere, such as ghost sizing or how evictions are recorded, rather than at where returners are placed in main.
