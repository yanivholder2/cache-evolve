# Variant v001 Insight

## Helped

(No dimension improved. This fitness vector reports only `hit-rate`. `metadata-size` and `per-op-time` were not measured, though both must be at least as bad as the parent's: the ghost now keeps up to twice as many keys, and `put` scans it linearly with `iter().any` and `retain`.)

## Hurt

- hit-rate: Doubling `ghost_size` to `2 * main_size` keeps small-queue rejects in memory roughly twice as long, so more of them count as ghost hits. Each ghost hit is pushed into main as a new entry with freq 0, and when main is full that first calls `evict_main`, which removes a resident main entry. Keys rejected that long ago are weak reuse candidates, and on workload-a re-admitting them cost more hits by displacing proven main residents than it gained.

## Takeaway

This variant changed only the ghost size and kept the 10% split fixed, and hit-rate dropped slightly. So a larger ghost is not what made the parent better, and the parent's gain should be credited to the larger main queue rather than to the larger ghost that came with it. With unconditional ghost-to-main re-admission at freq 0, a longer ghost history works against main's residents on workload-a. Ghost size should go back to `main_size` or smaller. If ghost size is revisited, it should be tested together with a stricter re-admission rule, such as entering at the main tail or requiring a second hit.
