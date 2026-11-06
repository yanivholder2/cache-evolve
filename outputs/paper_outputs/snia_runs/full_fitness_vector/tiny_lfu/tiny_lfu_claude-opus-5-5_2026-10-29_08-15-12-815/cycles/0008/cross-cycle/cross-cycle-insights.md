# Cross-Cycle Insights (through cycle 0008)

## Recurring wins

- **Orthogonal merges add up.** Every merge of changes that touch separate components (residency/window-SLRU, sketch storage/aging, hashing, link layout, update rule) has stacked without interference from cycle 0002 through 0006. Merging winners from the previous cycle is a reliable default for the next one.
- **Bit-packing in data-only structures pays off.** Nibble-packed 4-bit counters, cache-line-blocked sketch rows, sentinel-tagged u32 links, and the 24-byte node all cut metadata. Once codegen was fixed (cycle 0005), none of them cost latency. Packing works when it replaces storage. It fails when it adds read-modify-write work to hot stores (see failures).
- **Hit-rate levers split by cache size, and complementary pairs win.** Window size and window floor help the small size. Conservative update (fixing correlated in-block overestimation) helps the large size. Combining them was a net gain each time.
- **Approximation is fine where exactness is not load-bearing.** A single cheap hash, pairwise (not exact) conservative-update min, and nibble counters all kept accuracy at lower cost.
- **Codegen and instruction-level fixes beat algorithmic latency theories.** The two biggest latency wins after cycle 0002 were u32-throughout residency without bounds checks (0005) and forced inlining of `increment_hashed` (0006). Both removed costs that had been blamed on the algorithm.
- **Explicit, bounded admission bonuses work.** The guarded `>= 2` tie-break for re-seen candidates (0008) gained in every cell. It is the first lever to help small and large sizes at once.

## Recurring failures

- **Software prefetch on the inlined line.** Sketch-block and neighbour-node prefetch either overlapped with the inlining gain or could not break the pointer chase, because the links live inside the node that has to be loaded first. Three attempts at the get-p99 pointer-chase theory have failed (sketch prefetch, neighbour prefetch, in-link CLOCK bit).
- **Flags packed into link words.** Packing flags into links turns blind relink stores into load-then-store and multiplies relinks. This regressed both latency and hit-rate.
- **Runtime adaptivity (hill-climbing window).** It cost latency on every op through bookkeeping in `get`, and its gain may have come from bypassing the frequency contest.
- **Uniform window-percentage increases.** These help the small size and cost the large size. Use an absolute floor instead.
- **Rearranging counters within a block.** Moving counters around cannot reduce load per counter, and it breaks the non-aliasing row-to-word-pair layout that lets RMWs run independently.
- **Latency theories built on the update rule.** Branch removal and shortening the min chain did not help. The real cost was the out-of-line call.
- **Removing implicit "seen again" signals without a replacement.** Dedup of the miss→put double count alone hurt the large size. It was acting as both an aging-clock driver and a tie-break edge.
- **Single-run per-op-time verdicts.** Diffs with byte-identical hot paths have swung latency sharply in both directions (cycles 0004, 0005, 0007, 0008). Treat any single-run latency verdict as suspect.

## Open questions

- Does the `>= 2` tie-break stack with the clock-preserving `tick()` (count skipped increments toward `additions`), or does restoring the clock undo the small-size gain?
- What is the best tie-win threshold, and does it hold at other cache sizes?
- What actually drives get p99? Candidates: a slot-indexed reference bitset outside `Node`, or link/region data stored next to the map value.
- Can the benchmark be made trustworthy, with interleaved A/B or repeated runs, so that latency verdicts mean something?
- Are there residual large-size gaps (e.g., hm_0 dip under conservative update) that need more counters per addition rather than a different placement?
- Why are window-floor gains trace-specific (hm_0 moves, web_0 barely does)?

## Per-policy notes

- **W-TinyLFU (window + SLRU):** This is the backbone of every winning line since cycle 0002. Size the window by an absolute floor, not a percentage. Promotion on protected hits is the suspected get-p99 source. Avoid CLOCK-in-link variants.
- **Count-min frequency sketch:** Keep it cache-line-blocked, with 4-bit counters, a fixed row-to-word-pair layout, a single hash, aging by halving, pairwise conservative update, and `increment_hashed` forced inline. Do not tune `SAMPLE_FACTOR` or counter width for the large-size slip, since conservative update explains and fixes it. Changes to how often counts are added also shift the reset rate.
- **Admission contest:** Strict `>` disadvantages newcomers. A bounded tie-break for re-referenced candidates (`>= 2`) is the current best. Keep the frequency contest on all fill paths.
- **Residency layout:** The 24-byte u32-link node with sentinel encoding is the base. It has no latency cost once indices stay u32 and bounds checks are removed. Keep hot-path link stores blind (no extra flag bits).
