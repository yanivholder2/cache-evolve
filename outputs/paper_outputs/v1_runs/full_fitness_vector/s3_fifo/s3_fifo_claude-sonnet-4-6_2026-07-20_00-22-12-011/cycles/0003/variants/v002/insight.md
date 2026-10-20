# Variant v002 Insight

## Helped

- hit-rate: Shrinking the small queue from `capacity/10` to `capacity/20` gave main ~95 % of capacity instead of 90 %, allowing more hot Zipfian keys to reside simultaneously in the resident queue and reducing cold-key interference with them.
- per-op-time: The smaller small deque (102 vs 204 slots at capacity=2048) shortens the linear key scan in `get()`, cutting the worst-case scan length in half and lowering get p99 latency.

## Hurt

(none)

## Takeaway

Halving the small-queue fraction continued the monotone trend established by previous cycles: a narrower admission filter reduces cold-key residency time, freeing more of the fixed capacity for the hot working set and delivering a consistent hit-rate gain on Zipfian workloads. The get p99 improvement is a structural side-effect — the linear scan through `small` in `get()` is bounded by `small_size`, so halving the admission queue directly halves the worst-case scan depth. Both benefits came from a single one-line arithmetic change with no algorithmic risk.
