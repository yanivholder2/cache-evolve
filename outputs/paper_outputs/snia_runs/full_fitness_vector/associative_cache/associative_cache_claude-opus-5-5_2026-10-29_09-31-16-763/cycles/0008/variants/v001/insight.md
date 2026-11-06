# Variant v001 Insight

> Source note: the overlay source files could not be listed or opened in this session (the only tools available were Read and Write, and none of the guessed paths existed). This insight is based on the H01 change description and the fitness vectors of v001 and its parent (cycle 0007 v003). It has not been checked against the actual diff.

## Helped

- hit-rate: Not a gain, but the invariant held. Every trace/size cell matches the parent, which is consistent with the hand-written read/copy/write shift producing the same post-hit `items`/`tags` order as `remove` + `push`. The hit path also writes no `Item` fields, so eviction order is unchanged.
- metadata-size: Flat as designed. The change only touches control flow on the hit path and adds no struct fields, so `Item<u64,u64>` and the cache handle keep their sizes.

## Hurt

- per-op-time: Get latency got much worse at both p50 and p99, the opposite of what H01 predicted. Replacing the policy's `remove` + `push` with a raw `ptr::copy` of length `n - 1 - position` (a runtime value) most likely turns the MRU move into an out-of-line `memmove` call per hit, plus a second one for `tags`. The parent's code let the compiler emit a short inline move for the small, bounded way count.
- per-op-time: Put also slowed down sharply at p50 and p99, even though only the update-on-hit branch of `put` was meant to change. Get and put degrading by a similar factor points to a cost that both paths share. Likely candidates are the shared shift helper or the `unsafe` block blocking inlining or vectorisation of the surrounding bucket loop. Code that runs only on hits is unlikely to cause a regression this broad.

## Takeaway

Hand-writing the one-slot MRU shift with `ptr::read`/`ptr::copy`/`ptr::write` kept the semantics exact but made both get and put clearly slower. Devirtualising the move does not help when the replacement primitive is a variable-length `memmove`. On an 8-way-scale bucket, the move needs to stay inlinable: either a fixed-width copy (or a const-generic way count), or a fused swap loop like the one that won on put in earlier cycles. The regression is much larger than the run-to-run p99 noise seen in the parent, so before abandoning the idea, check the generated assembly for a `memmove` call and a de-inlined `get_from_position`. Also check that the MRU-skip branch does not mispredict on the benchmark's access mix.
