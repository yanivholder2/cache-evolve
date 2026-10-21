# Variant v001 Insight

## Helped

(No measured dimension improved. Only `hit-rate` is in the fitness vector. The intended savings are real but were not measured. Each bucket's ghost ring drops from four `Option<K>` slots to two. The miss-path scan in `admission_counter` drops from four compares to two.)

## Hurt

- hit-rate: Halving `GHOST_LEN` means a bucket now forgets an evicted key after just two more evictions. Keys that come back at eviction distance 3–4 used to find their ghost and get admitted with counter 1. Now they miss the ring, are admitted with counter 0, and the LIP-style hand that rests on the newcomer evicts them again on the bucket's next miss. Hit rate fell just below the 0.731 threshold that H01 set, so the hypothesis is refuted, but only narrowly.

## Takeaway

On workload-a at 2048, returners at ghost distance 3–4 still carry some useful signal, but not much. Halving the ring cost only a small slice of the gain v003 got from its 4-entry ring. So most of that gain comes from the two most recent evictions, and 4 sits at or near the flat top of the ring-size curve (cycle 0005 already showed 16 regressing). A 3-entry ring is the only size left untested between 2 and 4. Unless per-bucket metadata or miss-path cost becomes a measured objective, the extra two slots are worth keeping.
