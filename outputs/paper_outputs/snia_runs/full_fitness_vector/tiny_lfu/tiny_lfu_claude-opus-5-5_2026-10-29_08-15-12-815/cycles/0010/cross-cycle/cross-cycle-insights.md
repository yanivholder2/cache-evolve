# Cross-Cycle Insights

## Recurring wins

- Merges of levers that touch separate layers keep adding up with no interference (cycles 0002–0006, 0010). Examples: window/SLRU with fast hash, the aging sketch with residency, u32 links with the blocked sketch, and the window floor with pairwise conservative update. When two winners change different parts of the cache, merge them first.
- A safe hit-rate lever moves only one cache size. Conservative update and capacity-gated slower aging are clean examples. Levers that act on both sizes tend to help one and hurt the other.
- Small-size hit-rate responds to admission and recency levers: a larger window or window floor, and slower aging (cycles 0004, 0005, 0008–0010). Slower aging is the confirmed lever and has not saturated yet.
- Large-size hit-rate responds to the quality of frequency evidence. Conservative update fixed correlated in-block overestimation (cycles 0003–0004). The pairwise min keeps nearly all of its accuracy (cycle 0005).
- Packing bits into compact metadata paid off: nibble counters, sentinel-tagged u32 links and a 24-byte node (cycles 0002–0005). The latency cost of the compact layout went away once the codegen was fixed.
- The real latency wins came from codegen and data structures, not from policy changes. Examples: a single cheap hash, the cache-line-blocked sketch, removing bounds checks, `#[inline(always)]` on the increment, and the packed linear-probing index (cycles 0001, 0002, 0005, 0006, 0010).
- Splitting a coupled change into its separate variables made the cause clear (cycle 0008). The miss double-count turned out to act as an implicit "seen again" bonus.

## Recurring failures

- Software prefetch hints, on the sketch or on list neighbours, add nothing once the increment is inlined. The node has to be loaded before its neighbours are known, so prefetch cannot get ahead of the pointer chase (cycles 0006–0007).
- Explaining the get tail as a list-link or relink cost failed four times: neighbour prefetch, a CLOCK flag in `next`, a CLOCK reference bit in a separate bitset, and the promotion path. Bits packed into link words turn blind stores into load-then-store. Drop CLOCK and reference-bit replacements for protected LRU.
- Bonuses for the window candidate overlap instead of adding up. The double count, the tie-break and the restored tick all fix the same large-size newcomer loss. Stacking any two of them over-favours recency at the small size (cycle 0009).
- Making the victim colder works like a candidate bonus: it loosens the filter. Under LRU probation, "colder by estimate" mostly means "young and not yet re-referenced" (cycle 0010).
- These diagnoses were wrong or gave nothing: branch removal alone, shortening the `min` chain, uniform window-percentage increases, in-block counter re-addressing, uniform ticks, and runtime hill-climbing windows.
- Single-run per-op-time verdicts are unreliable. Byte-identical hot paths swung both ways in at least five cycles (0004, 0005, 0007–0010). Read hit-rate and latency verdicts separately, and trust a latency result only when it is consistent across all four latency stats or has a clear mechanism.

## Open questions

- Where is the ceiling on small-size aging slowdown? 4-bit counter saturation is the suspected limit. Would a mild slowdown at the large size also help there?
- Can put p99 under the packed index be fixed, through a fused remove+insert on eviction or a lower load factor, without losing the get gain? Does the index gain hold up in interleaved runs and stack with the aging change?
- Should the tie-break threshold, the aging factor and the window floor scale with capacity or sketch width instead of being constants?
- Would an age-aware or stale-frequency victim signal succeed where raw estimate comparison failed?
- What is left of get p99 after the index change? The inlined sketch update is the main remaining suspect.
- Can the benchmark be made stable with interleaved A/B and repeat runs, so that latency effects near the noise floor can be judged?

## Per-policy notes

- **W-TinyLFU (window + SLRU), the current line:** The base is the 24-byte u32-link node, the blocked 4-bit aging sketch, pairwise conservative update with forced inlining, a size-aware window floor, miss dedup with no tick, a `>= 2` tie-break, and capacity-gated slower aging. The packed index is pending merge. Keep the protected segment as strict LRU.
- **TinyLFU sketch:** Keep the cache-line-blocked layout with fixed row-to-word-pairs, since rows that never alias let the updates run independently. Fight overestimation with conservative update, not with placement inside the block. Any change to how often increments are added also changes the aging clock and tie behaviour, so keep the two coupled in mind.
- **Admission contest:** The strict `>` test underserves re-seen newcomers. Use one explicit, bounded bonus, not several.
- **Adaptive window (hill climber):** It cost latency on every op, and its gain may have come from bypassing the frequency contest. Revisit only with the contest restored and a step floor.
- **Map/index:** The hashbrown index was a hidden latency cost. Linear probing helps get, and its eviction-path deletion is the new put-tail cost.
