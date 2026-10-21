# Variant v003 Insight

## Helped

- hit-rate: Placing first-access items in the cold region and promoting only on the second hit keeps one-touch scan items from displacing reused items, which the parent's promote-on-every-hit LRU does not do.
- per-op-time: A first hit now does nothing, where the baseline did remove-and-promote. Dropping the per-get statistics and timestamp writes also shortens the get path. Both p50 and p99 improved for get and put.

## Hurt

- metadata-size: No change. The seen-once flag fits in spare bits of the existing header, so the item stays the same size and nothing was gained or lost here.

## Takeaway

Second-hit admission with the flag packed into existing header bits improved hit-rate and per-op-time together without growing metadata. The policy gain and the removal of unread bookkeeping cannot be separated from these results, because both changes landed at once. I could not read the overlay source in this session, so this is inferred from the hypothesis and the fitness vectors.
