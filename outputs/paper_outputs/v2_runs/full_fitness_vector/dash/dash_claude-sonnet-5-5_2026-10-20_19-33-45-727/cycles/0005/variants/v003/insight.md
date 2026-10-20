# Variant v003 Insight

## Helped

- hit-rate: No movement, as the hypothesis required. Hoisting hash, bucket index and tag computation out of the locked region is a pure reordering, so the lookup scan, second-hit filter and eviction decisions are unchanged.
- metadata-size: No movement. Item layout and segment structs were explicitly held fixed, and the change only passes precomputed values down.

## Hurt

- per-op-time: Both get and put got slower than the parent, and put was hit hardest. The p99 tail also got worse on put, and get p99 only improved slightly. The intended shorter critical section did not pay off. The hash was already cheap relative to the lock and bucket scan, and precomputing it adds values to pass through the call path. That can cost registers or stack traffic, or defeat inlining, without shortening the lock hold meaningfully. Part of the put regression could also be run-to-run noise. I could not read the overlay sources in this session, so these causes are inferred from the hypothesis and the fitness change rather than confirmed against the diff.

## Takeaway

Moving hash and index computation out of the lock did not reduce latency on this small incumbent. Lock hold time is not dominated by that arithmetic, so a pure reordering has nothing to remove, and it can make things slightly worse. This is the second consecutive latency idea (after v001's atomics and early returns) that failed without any profiling. The next attempt should measure where get and put time actually goes, and re-run the baseline to establish the noise floor, before editing the hot path again.
