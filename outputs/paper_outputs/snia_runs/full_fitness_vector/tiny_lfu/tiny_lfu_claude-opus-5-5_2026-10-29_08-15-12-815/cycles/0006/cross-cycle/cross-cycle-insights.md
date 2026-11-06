## Recurring wins

- Merging separate winners has stacked additively every cycle (0002–0006): residency structure, sketch storage, link compaction, conservative update and window sizing combined without cancelling. Merging the previous cycle's winners is a reliable first move.
- Bit packing pays off over and over: sentinel-tagged links, nibble counters and u32 indices shrank metadata. Once the codegen was fixed (0005), it cost nothing in latency or hit-rate.
- Cache-line locality in the sketch (blocked layout, fixed row-to-word-pair mapping with no aliasing) fixed the scattered-miss get tail and lets the row RMWs run in parallel.
- The hit-rate levers split by cache size. The window, the window floor and admission-structure changes help the small size. Conservative update (exact or pairwise) helps the large size by fixing correlated in-block overestimation. Combining them works.
- An approximate update rule is good enough: pairwise conservative update keeps nearly all of the exact-min accuracy.
- Recent latency wins have come from codegen and instruction ordering, not from algorithm changes: a single cheap hash, prehashed map keys, `#[inline(always)]` on the increment, u32-native index paths without bounds checks, and prefetching plus putting the probe before the increment.

## Recurring failures

- Hit-rate changes that forget or bypass frequency evidence (aging, adaptive window, uniform window growth) keep slipping slightly at the large size.
- Runtime adaptivity (the hill-climbing window) added latency on every op, and its gain was confounded by window fills that skipped the frequency contest.
- Using micro-structure of the update rule as a latency lever did not work: branch-free masks and a shorter min chain recovered nothing. The real cost was the out-of-line call.
- Rearranging counters inside a fixed sketch block cannot lower load per counter, and doing so hurt put latency. Placement inside the block is a dead end.
- Tuning the aging period or counter width for the large-size slip was the wrong diagnosis.
- Per-op-time verdicts from single runs have repeatedly been noise: byte-identical hot paths slowed down, and so did unrelated tails. Latency claims need interleaved or repeated A/B runs.

## Open questions

- Get p99 is the one stubborn metric. Is it caused by `on_hit` probation→protected promotion link misses? Would prefetching or reordering the neighbour nodes fix it?
- Does the prefetch/probe reorder stack on top of the inlined merged line, or do the two get-p50 gains overlap?
- Large-size hit-rate is near its ceiling with the current counter layout. Further gains need more counters per addition. Is the small large-size dip on hm_0 a conservative under-counting ceiling or noise?
- Why is the window-floor gain trace-specific (hm_0 gains, web_0 flat)?
- The benchmark noise floor is near the size of the effects being measured, so a stable interleaved harness remains a prerequisite.
- Unmeasured costs remain: per-entry stored hashes and growth of the fixed cache struct are not in the metadata metric.

## Per-policy notes

- W-TinyLFU (window + SLRU main): this is the backbone of every winning line. Size the window as an absolute floor (max of the percentage and a fixed entry count), not as a larger uniform percentage. Do not use an adaptive climber unless the frequency contest is kept.
- Count-min admission sketch: blocked 64-byte layout, 4-bit packed counters, periodic halving, pairwise conservative update, force-inlined increment. Keep the fixed row-to-word-pair mapping.
- Residency layer: the 24-byte node with u32 sentinel-encoded links is the base. Its old latency regression was codegen and is now resolved. The earlier hit-rate gap against the 40-byte line came from missing levers, not from `reset()`.
