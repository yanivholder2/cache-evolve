# Variant v003 Insight

## Helped

- hit-rate: No dimension moved — adding the second salted home (`STASH_ALT_SALT2` / `stash_alt_index2`, with `stash_insert` trying it whenever the first alternate is full or non-distinct and `promotion_get` searching all three homes) rescued no keys the single-hop parent was already dropping.

## Hurt

- hit-rate: The breadth lever paid for an extra salted index computation per overflow victim and per stash-hit lookup, yet the second alternate either lands on a key the cache reclaims through promotion anyway or is itself full at the same moments the first one is, so the widened home-set produced an exactly flat result.

## Takeaway

Widening the stash home-set from two homes to three does not lift hit-rate on top of the v001 winner: the premise that "a second non-full alternate exists more often than the first" does not convert into retained keys here, because the buckets that block the first alternate tend to block the second at the same time, and the marginal dwell-time it could buy covers keys the two-speed promotion already protects. Breadth-of-homes is as inert as the single hop was — this seam overlaps the protected-tier seam rather than stacking with it.
