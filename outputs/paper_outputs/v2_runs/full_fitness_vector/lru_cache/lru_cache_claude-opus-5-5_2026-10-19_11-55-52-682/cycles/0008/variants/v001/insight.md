# Variant v001 Insight

## Helped

- hit-rate: Halving the direct-mapped ghost (`gsize = next_power_of_two(capacity) / 2`) produced exactly the same gain on the VISITED-bitset layout as it did on v002's layout in cycle 0007's v003. This supports the hypothesis that ghost recall and VISITED storage are independent. The likely mechanism is that a smaller ghost forgets old small-queue evictions sooner. Under this trace, ghost hits are then limited to keys that return quickly, so fewer stale one-touch keys are admitted straight into main, where they would displace SIEVE-protected entries.

## Hurt

- per-op-time (`get` p50/p99): Regressed against the parent, which falsifies the hypothesis's prediction that `get` p50 would stay at or below 27 ns. The code does not explain this. `get` never reads `ghost` or `gmask`, and its hit path (map probe, `visited` bitset, `nodes[idx]`) is byte-for-byte identical to the parent's. The regression is most likely a change in the benchmark run, not an interaction between the layouts.
- per-op-time (`put` p50/p99): Also regressed, even though the change should make `put` slightly cheaper or leave it unchanged. The miss path still does one ghost probe and, on a small eviction, one ghost write, but now into a 4 KiB table instead of an 8 KiB one. Every latency figure got slower by a similar proportion, including `get`, which does not touch the ghost. That points to measurement drift, not to the smaller ghost.

## Takeaway

The half-size ghost improves hit-rate the same way on two different node layouts, so it can be treated as a layout-independent policy gain and kept in the stack. Every latency figure got slower, including `get`, whose code path did not change. These latency numbers should not be read as a cost of the ghost change. Benchmark both this variant and its parent again in the same run before deciding whether the two changes interact.
