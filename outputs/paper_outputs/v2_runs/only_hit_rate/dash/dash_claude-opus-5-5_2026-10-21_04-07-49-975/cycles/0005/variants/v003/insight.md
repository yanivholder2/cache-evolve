# Variant v003 Insight

> Note: I could not list or open the overlay source in this session; the only tools available were file read and write, with no directory listing. The mechanism below comes from H03's specified changes. The diff itself was not checked.

## Helped

- hit-rate: Under the parent's BIP rule, a fresh key returning after a long reuse distance always landed in the stash's one-slot LRU probation area, where the next insert evicted it before it could be reused. The per-segment ghost table holds fingerprints of keys evicted from the stash, covering about one cache's worth of evictions. When a returning key matches its ghost fingerprint, it is inserted at the stash MRU end. It then survives the second-chance scan long enough to collect a hit. One-touch scan keys never appear in the ghost table, so they still churn through probation. The gain is small but positive. This suggests that only a modest fraction of misses on this trace come from keys whose reuse distance falls inside the ghost window but beyond the parent's probation slot.

## Hurt

## Takeaway

A ghost window that changes where a returning key is placed works where cycle 0004's ghost attempt failed. That attempt used a tiny per-bucket ghost window and only pre-credited the counter. This variant confirms the parent's lesson that insert position, not counter credit, is the lever, and shows that "seen recently enough to come back" is a usable signal for MRU placement. The small size of the gain suggests the remaining misses are mostly true one-touch keys or reuse distances longer than one cache's worth of evictions. Two things are untested here: a larger ghost window, and letting a ghost hit also protect against the next scan pass. The added metadata (2 bytes × `GHOST_SLOTS` per segment) is not captured by any dimension in this fitness vector, and the result is from a single LIRS trace at a single cache size.
