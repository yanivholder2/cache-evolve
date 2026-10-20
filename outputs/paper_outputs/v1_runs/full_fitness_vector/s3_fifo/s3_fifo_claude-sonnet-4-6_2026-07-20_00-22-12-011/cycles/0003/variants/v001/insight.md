# Variant v001 Insight

## Helped

- per-op-time: Replacing the second-chance loop's repeated `pop_back` + `push_front` churn with a single `swap` + `pop_back` eliminated the deque memory-layout disruption that was inflating get tail latency, cutting get p99 from 200 ns to 100 ns.

## Hurt

- hit-rate: Two-random-choices evicts whichever of two randomly sampled entries has lower frequency unconditionally, so hot entries with frequency ≥ 1 can be discarded on a single bad draw; the second-chance loop they replaced would have cycled past those entries and preserved them until their frequency drained to zero, making the hit-rate drop material and not merely a rounding effect.

## Takeaway

Replacing the O(n) second-chance scan with O(1) two-random-choices is a speed–quality trade-off that favors latency at the cost of eviction precision: the deque churn from second-chance cycling was the source of get tail-latency jitter, and eliminating it halved p99, but the policy change itself removes the frequency-drain protection that kept hot items resident — on workload-a, where the hot set is large relative to main capacity, that protection was worth more than the algorithmic speed-up.
