# Variant v001 Insight

## Helped

- per-op-time: Restoring MAX_FREQ to 3 caps each main-queue entry's second-chance loop at three decrements instead of five, reducing put p99 latency (400 ns in the parent to 300 ns here) because evict_main() reaches a zero-frequency entry in fewer iterations on average.

## Hurt

- hit-rate: Lowering the second-chance ceiling from 5 to 3 reduces how many times a hot main-queue entry can defer eviction, confirming that MAX_FREQ=5 did provide a small but real additional protective window for the Zipfian hot set.

## Takeaway

Rolling MAX_FREQ back from 5 to 3 delivers the expected put tail-latency improvement — the shorter evict_main() scan is measurable — but the per-op-time summary score (anchored to get p50) does not move, so the gain is invisible to the primary fitness signal. The hit-rate slip is real but tiny, confirming the parent's insight that MAX_FREQ is a near-exhausted lever on this Zipfian workload. If the goal is to recover the per-op-time summary, the get path rather than eviction scan depth needs attention.
