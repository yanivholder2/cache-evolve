## Recurring wins

- Merges of winners that each change one part of the cache keep adding up (cycles 0002 and 0003). Fast hash plus window/SLRU, window/SLRU plus the blocked aging sketch, and compact links all combined with no interference on residency. Merging the previous cycle's winners first is a dependable opening move.
- Bit packing and sentinel encoding keep paying off: tagged links removed the region byte, nibble-packed counters shrank the sketch and made aging cheap, and u32 links cut metadata. Each kept hit-rates the same or better while lowering metadata.
- Memory-locality fixes deal with tail latency. Single-hash keys, a prehashed map key, and the cache-line-blocked sketch together resolved the get-tail weakness flagged in cycle 0001.
- Fixes to admission and frequency quality give most of their hit-rate gain at the small cache size. Window/SLRU, aging, and conservative update each improved the small cache most.
- Conservative update is the clearest causal fix for the large-cache slip. That slip came from correlated in-block overestimation, not from the aging period or counter width.

## Recurring failures

- Per-access work in `get` keeps costing median latency. Examples are u32-counter aging (cycle 0001), hill-climber bookkeeping (cycle 0002), and the saturation and sample checks plus branchy two-pass conservative update (cycle 0003). The bottleneck is control flow, not memory.
- Runtime adaptivity (the hill-climbing window) cost latency on every op. Its small-cache gain may have come from skipping the frequency contest, not from the window size itself. Shelved.
- Tuning `SAMPLE_FACTOR` or counter width for the large-cache slip is a dead end, because conservative update already explains and fixes it.
- Mechanisms that forget or bypass frequency evidence (aging, window fill without a contest) tend to slip a little at large capacity.

## Open questions

- Can a branchless, masked, word-level saturating conservative update keep its accuracy at close to single-pass increment latency?
- Does conservative update still give its large-cache gain on top of window/SLRU, or does the window absorb part of it?
- Is the latency regression from compact u32 links real, for example from codegen or bounds checks, or is it noise? It needs a back-to-back, same-session benchmark.
- Costs that are not yet measured: the per-entry stored hash in the map and the growth of the fixed cache struct. Neither appears in the metadata metric.
- Does the merged cache still leave a gap at the small size that could justify bringing back adaptivity with the frequency contest restored?

## Per-policy notes

- **W-TinyLFU (window + SLRU):** This has been the dominant residency structure since cycle 0001. It is stable under merges, and its hit-rate gain is concentrated at the small size. Sentinel-tagged and u32 links keep its metadata low.
- **Count-min / frequency sketch:** It has evolved from a SipHash multi-hash with u32 counters to a single cheap hash, then a cache-line-blocked layout, packed 4-bit counters, periodic halving, and conservative update. Accuracy problems are now solved. What remains is the increment's per-op cost.
- **Adaptive window (hill climber):** It tested negative once. Revisit only with the contest restored, a step floor, and evidence of a remaining small-cache gap.
- **Baseline strict admit-or-drop without aging:** This was the original source of hit-rate loss at small capacity and has been fully superseded.
