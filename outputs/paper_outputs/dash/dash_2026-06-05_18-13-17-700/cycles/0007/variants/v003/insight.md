# Variant v003 Insight

## Helped

- hit-rate: Replacing v002's single relocation hop with a bounded (3-hop) cuckoo cascade in `stash_insert` rescues overflow victims whose immediate alternate home is also full — by displacing that bucket's LRU resident and chasing free space a few buckets deeper, more cold keys survive long enough to catch their second reference instead of dropping on the first full alternate.

## Hurt

- (none): No measured dimension regressed. Occupancy stays conserved because exactly one item is carried homeless and at most one is dropped per insert (each `put_at` into a full bucket evicts one, seats one), and `RELOCATION_MAX_HOPS = 3` caps per-op cost and guarantees termination even when keys ping-pong between their two homes.

## Takeaway

The marginal gain v002 left on the table was real dwell-time headroom: most of its single-hop rescues died because the one alternate home was full. Letting the rescue cascade through a few legal homes — evicting an LRU resident at each full bucket and carrying it onward — converts those would-be drops into survivors, lifting hit-rate without inflating capacity. Depth, not just a second home, is what unlocks the stash's slack.
