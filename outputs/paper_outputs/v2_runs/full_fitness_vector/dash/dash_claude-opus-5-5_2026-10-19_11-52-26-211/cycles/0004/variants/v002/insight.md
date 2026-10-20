# Variant v002 Insight

## Helped

- hit-rate: `stash_insert` now puts an item into the free slot of `s1 = (s0 + 1) % stash_size` when the home bucket `s0` is full, and only evicts from `s0` when both are full. So a stash bucket that overflows can use the hole that a stash hit's `remove_at` leaves in its neighbour, and it no longer has to drop an item. Hit rate rose above the 0.702 bar the hypothesis set, so it is confirmed: on workload-a, holes in 2-slot stash buckets are not refilled by their own home keys fast enough to make a spare second choice useless.

## Hurt

- per-op-time: `get` p50 rose clearly, by more than the "slightly" the hypothesis predicted. `promotion_get` checks the stash first, so every miss, and every key that is not in `s0`, now scans a second 2-slot stash bucket through `find_in_stash` before it reaches target and probing. Misses and stash hits make up a large share of gets, so this extra scan shows up at the median, not just in the tail. `put` p50 and p99 also rose. Every stash insert, whether a fresh put or one of the two demotion paths, now computes both indices and may check whether a second bucket is full before it picks one. `get` p99 hardly moved, which fits with the worst-case path (the promotion cascade) being unchanged.

(metadata-size did not change. Two-choice placement is computed from the key hash on every operation, so no per-item field or flag was added, and `Item` stays at 40 bytes.)

## Takeaway

Making the stash 2-way associative, with fill-before-evict in the next bucket, gains hit rate without any new metadata. This confirms that stash associativity was a real limit on top of v003's probationary-stash ordering. The cost is that the stash comes first in lookup order, so every miss pays for the second bucket scan. A later variant should try to recover `get` latency without giving up the two-choice gain, for example by checking target/probing before the stash, or by filtering the `s1` scan. Fixed-stride neighbour choice (`s0 + 1`) was enough here. Whether an independent second hash would retain more items is still an open question.
