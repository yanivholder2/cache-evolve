# Variant v002 Insight

## Helped

- hit-rate: A key evicted within its bucket's last 4 miss-path evictions now comes back at index 3 instead of index 1 in a full 16-slot bucket. That gives a key that has already shown reuse about two more misses to be hit again before it is evicted. Ordinary newcomers still enter at index 1, so the fast removal of one-off keys that the parent won is kept. The gain over the parent is small, only just past H02's threshold, and smaller than the parent's own step from 1/4 to 1/8.

## Hurt

(None measured. This evaluation recorded only hit-rate. The change adds a 4-key ghost ring plus a `u8` cursor to every bucket, so metadata-size must have grown. Each miss also adds a linear scan of 4 ghost slots, and every eviction adds a ghost write, so a small per-op-time cost is expected. Neither cost was recorded, so both are unconfirmed.)

## Takeaway

The ghost signal does carry over from CLOCK to the shift-based 1/8 LRU line when the reward is capped. Moving a returning ghost two slots deeper is a net positive. The hypothesis says the uncapped MRU reward in cycle 0002 hurt, and this capped reward helps, so the size of the reward matters more than whether ghosts are used at all. The small margin fits H02's alternative explanation: the one slot behind a 1/8 newcomer already absorbs most short-reuse keys, so the ghost ring mainly rescues keys whose reuse distance is just beyond the bucket. Ghosts are recorded only when a full bucket evicts, so ghost hits always land at a bucket length of 15. That means the 1/4 depth is in effect a fixed index 3, not a proportional depth. One trace at one size cannot show whether this margin is above run-to-run noise.
