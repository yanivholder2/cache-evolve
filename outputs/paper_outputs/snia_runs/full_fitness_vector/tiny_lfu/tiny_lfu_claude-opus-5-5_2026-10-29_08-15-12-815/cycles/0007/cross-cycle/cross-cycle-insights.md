## Recurring wins

- **Merging single-axis winners adds up.** Every merge from cycle 0002 to cycle 0006 combined cleanly: window/SLRU with fast hash, window/SLRU with the aging sketch, the aging sketch on 24-byte nodes, and the window floor with pairwise conservative update. Residency changes and sketch changes have not interfered with each other.
- **Bit packing pays off.** Nibble counters, sentinel-tagged links and u32 links each cut metadata and caused no lasting latency cost once their codegen was fixed.
- **Cache-line-blocked sketch with a fixed row-to-word-pair layout.** It removed scattered misses (cycle 0002). Because rows never alias, the four updates run independently (cycle 0006). Keep it.
- **Conservative update is the lever for the large cache size.** It fixed the correlated in-block overestimation behind the large-size slip (cycles 0003–0004). A cheap pairwise min keeps almost all of its accuracy (cycle 0005).
- **The window is the lever for the small cache size.** A larger window, or a window with an absolute floor, raises hit-rate at the small size. Sizing it with a floor avoids taking slots from the main region at the large size.
- **Codegen fixes keep beating algorithm changes on latency.** Examples are single-hash, prehashed map keys, no bounds checks on u32 links, and `#[inline(always)]` on `increment_hashed`. Several apparent "algorithmic" latency costs turned out to be inlining or codegen artifacts.

## Recurring failures

- **Runtime adaptivity (hill-climbing window).** It added cost to every op and gained hit-rate only at the small size, partly through a path that skipped the frequency contest.
- **Fixing latency by reshaping the min logic.** Going branch-free (cycle 0004) and shortening the min chain (cycle 0005) both failed. The real cost was the out-of-line call.
- **Changing counter placement inside a sketch block.** It cannot reduce load per counter (cycle 0006). Overestimation has to be fought with more counters per addition or with conservative update.
- **Software prefetch on the inlined merged line.** Sketch prefetch overlaps with the inlining gain, and neighbour prefetch cannot break the pointer chase through the hit node (cycle 0007). It leaves hit-rate bit-identical and does not improve latency.
- **Changing one hit-rate knob without isolating its side effects.** Changing how often counts are added also changes the aging clock and the admission tie-break (cycle 0007 dedup). Changing the window percentage uniformly hurts the large size.
- **Trusting per-op-time from single runs.** Diffs with byte-identical hot paths regressed in cycles 0004, 0005 and 0007. The benchmark's noise floor is about the size of the effects being measured.

## Open questions

- Can get p99 be lowered by moving link/region data next to the map value, so promotion loads start without first loading the node? It is the one stubborn metric.
- Keep the get→put miss dedup but still count skipped increments toward `additions`. Does the small-size gain survive without the large-size loss?
- Does letting ties go to the candidate, or giving it a small bonus, recover web_0 at the large size? This would separate the recency-bias effect from the aging effect.
- Is the small hm_0 dip at the large size a real ceiling effect of conservative under-counting, or noise?
- Why does the window floor help hm_0 but barely move web_0? Is the gain specific to the trace?
- Repeated or interleaved A/B runs are needed to measure the benchmark's noise floor before more latency verdicts are made.

## Per-policy notes

- **W-TinyLFU (window + SLRU):** This is the base line. The current best stack is the 24-byte u32-link nodes, a window with an absolute floor, the blocked 4-bit aging sketch with pairwise conservative update, and an inlined increment. The small and large cache sizes keep pulling in opposite directions: the small size wants recency and window capacity, while the large size wants accurate frequency evidence. Promotion from probation to protected is the likely source of get p99.
- **TinyLFU admission sketch:** The sketch's aging rate, the number of increments per access, and the strict `>` admission test are coupled. Tune them as one system. Do not tune `SAMPLE_FACTOR` or counter width to fix the large-size slip, because conservative update already explains and fixes it.
- **Plain admit-or-drop (original baseline):** This is superseded. Its losses came from admission contests at small capacity and from a sketch that never forgot old counts.
