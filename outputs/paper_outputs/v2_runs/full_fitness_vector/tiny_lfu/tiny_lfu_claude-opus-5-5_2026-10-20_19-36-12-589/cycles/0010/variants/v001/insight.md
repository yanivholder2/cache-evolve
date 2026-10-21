# Variant v001 Insight

## Helped

- metadata-size: Packing two 4-bit counters per byte (`counts.len() == SKETCH_DEPTH * width / 2`) halves the sketch heap again, as `heap_bytes()` reports. At cap=2048 that brings the table down to the L1D-sized footprint the hypothesis aimed for. The headline `Node<u64,u64>` summary doesn't move because the sketch is a fixed per-cache cost.

## Hurt

- hit-rate: The hypothesis set its own refutation test: if hit rate fell, the cap of 15 was binding. It fell, so the claim that "admission is decided by low counts only" is false on this trace. The sketch has no aging, so counters only grow. Hot residents and moderately hot newcomers both pile up at 15, and the strict `incoming_freq > victim_freq` duel then rejects the newcomer on the 15-vs-15 tie. Collision-inflated counters also reach the ceiling sooner. The `u8` cap of 255 never caused these ties.
- per-op-time: The smaller table did not make `get` faster. The median got slightly worse, and the `get`/`put` tails and `put` p50 all regressed. Each of the four row probes in `increment_hash` now adds a shift, a mask and a shifted add, and that extra work cost more than any L1 gain. The sketch's memory traffic was not the bottleneck on the `get` hot path, which fits the parent's finding that median `get` is limited by the map lookup and hashing.

## Takeaway

Going from `u8` to 4-bit counters is not free here. TinyLFU's usual nibble counters assume periodic halving, which keeps counts well below the cap. Without that aging, a cap of 15 creates saturation ties that the strict `>` duel resolves against newcomers, so hit rate drops. Nibble packing also made the per-probe arithmetic heavier without buying measurable cache-locality gains, so the sketch-footprint lever for latency is now used up. If 4-bit counters come back, they should arrive together with a reset/halving mechanism, not on their own.
