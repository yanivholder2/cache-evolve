# Variant v003 Insight

## Helped

- per-op-time: Dropping the xor/shift avalanche fold from `finish()` leaves one multiply plus a rotate on the hash path. `get` improved slightly, and `put` improved much more, in both median and tail. `put` hashes on both the lookup and the insert or evict path, so it saves more.
- metadata-size: The `LruCache` struct shrank. The hypothesis didn't target this, so it is probably a side effect of the simplified hasher or map state (my inference, since the source diff wasn't readable). Node size is unchanged, as expected from leaving the node layout alone.

## Hurt

- hit-rate: No change. The rotate-based high-bit selection spreads keys well enough that bucket behaviour does not alter eviction decisions. This is a neutral result, not a regression.

## Takeaway

The avalanche fold was a real cost on the segmented variant, mainly for `put`. A single Fx multiply with `rotate_left` to expose the high bits gives hashbrown enough mixing, so the fold can be dropped without hurting hit-rate. The hasher is therefore a cheap lever for speed, independent of the packed segment bit and promotion logic. Note that this variant was submitted for clean attribution, and the struct-size drop suggests the change touched more than the hash arithmetic.
