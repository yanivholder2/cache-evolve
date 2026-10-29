# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-29T09:02:55+00:00

## timeline

### 09:03:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Merging single-axis winners adds up.** Every merge from cycle 0002 to cycle 0006 combined cleanly: window/SLRU with fast hash, window/SLRU with the aging sketch, the aging sketch on 24-byte nodes, and the window floor with pairwise conservative update. Residency changes and sketch changes have not interfered with each other.\n- **Bit packing pays off.** Nibble counters, sentinel-tagged links and u32 links each cut metadata and caused no lasting latency cost once their codegen was fixed.\n- **Cache-line-blocked sketch with a fixed row-to-word-pair layout.** It removed scattered misses (cycle 0002). Because rows never alias, the four updates run independently (cycle 0006). Keep it.\n- **Conservative update is the lever for the large cache size.** It fixed the correlated in-block overestimation behind the large-size slip (cycles 0003–0004). A cheap pairwise min keeps almost all of its accuracy (cycle 0005).\n- **The window is the lever for the small cache size.** A larger window, or a window with an absolute floor, raises hit-rate at the small size. Sizing it with a floor avoids taking slots from the main region at the large size.\n- **Codegen fixes keep beating algorithm changes on latency.** Examples are single-hash, prehashed map keys, no bounds checks on u32 links, and `#[inline(always)]` on `increment_hashed`. Several apparent \"algorithmic\" latency costs turned out to be inlining or codegen artifacts.\n\n## Recurring failures\n\n- **Runtime adaptivity (hill-climbing window).** It added cost to every op and gained hit-rate only at the small size, partly through a path that skipped the frequency contest.\n- **Fixing latency by reshaping the min logic.** Going branch-free (cycle 0004) and shortening the min chain (cycle 0005) both failed. The real cost was the out-of-line call.\n- **Changing counter placement inside a sketch block.** It cannot reduce load per counter (cycle 0006). Overestimation has to be fought with more counters per addition or with conservative update.\n- **Software prefetch on the inlined merged line.** Sketch prefetch overlaps with the inlining gain, and neighbour prefetch cannot break the pointer chase through the hit node (cycle 0007). It leaves hit-rate bit-identical and does not improve latency.\n- **Changing one hit-rate knob without isolating its side effects.** Changing how often counts are added also changes the aging clock and the admission tie-break (cycle 0007 dedup). Changing the window percentage uniformly hurts the large size.\n- **Trusting per-op-time from single runs.** Diffs with byte-identical hot paths regressed in cycles 0004, 0005 and 0007. The benchmark's noise floor is about the size of the effects being measured.\n\n## Open questions\n\n- Can get p99 be lowered by moving link/region data next to the map value, so promotion loads start without first loading the node? It is the one stubborn metric.\n- Keep the get→put miss dedup but still count skipped increments toward `additions`. Does the small-size gain survive without the large-size loss?\n- Does letting ties go to the candidate, or giving it a small bonus, recover web_0 at the large size? This would separate the recency-bias effect from the aging effect.\n- Is the small hm_0 dip at the large size a real ceiling effect of conservative under-counting, or noise?\n- Why does the window floor help hm_0 but barely move web_0? Is the gain specific to the trace?\n- Repeated or interleaved A/B runs are needed to measure the benchmark's noise floor before more latency verdicts are made.\n\n## Per-policy notes\n\n- **W-TinyLFU (window + SLRU):** This is the base line. The current best stack is the 24-byte u32-link nodes, a window with an absolute floor, the blocked 4-bit aging sketch with pairwise conservative update, and an inlined increment. The small and large cache sizes keep pulling in opposite directions: the small size wants recency and window capacity, while the large size wants accurate frequency evidence. Promotion from probation to protected is the likely source of get p99.\n- **TinyLFU admission sketch:** The sketch's aging rate, the number of increments per access, and the strict `>` admission test are coupled. Tune them as one system. Do not tune `SAMPLE_FACTOR` or counter width to fix the large-size slip, because conservative update already explains and fixes it.\n- **Plain admit-or-drop (original baseline):** This is superseded. Its losses came from admission contests at small capacity and from a sketch that never forgot old counts.\n"
}
```

### 09:03:20 chat

I've written `cross-cycle-insights.md` in the cycle 0007 output dir. It has the four required sections in order, all as short bullets, and is built only from cycles 0001–0007.

**Recurring wins**
- Merging single-axis winners has always added up: no merge has interfered with another.
- Packing bits and keeping the cache-line-blocked sketch layout cut memory use without a lasting speed cost.
- Conservative update is what helps hit-rate at the large cache size. The admission window is what helps at the small size.
- Most latency gains came from compiler-level fixes (inlining, bounds checks), not from changing the algorithm.

**Recurring failures**
- Hill-climbing the window size at runtime cost speed on every operation.
- Reworking the min step to speed up the sketch increment didn't help: the real cost was a function call that wasn't inlined.
- Moving counters around inside a sketch block didn't reduce overcounting.
- Software prefetch stopped paying off once the increment was inlined.
- Changing one hit-rate setting also changed how fast the sketch ages and how admission ties break.
- Single benchmark runs are not reliable for speed comparisons: changes whose hot path didn't change still showed slowdowns.

**Open questions**
- Can get p99 be reduced by storing link data next to the map entry?
- Can the duplicate-increment removal be split into two separate tests: one on the aging clock and one on the admission tie-break?
- Is the hm_0 dip at the large size real, and why is the window floor's gain specific to the trace?
- What is the benchmark's noise floor? Repeated or interleaved runs would measure it.

**Per-policy notes** cover W-TinyLFU (the current best combination, plus the split between small and large cache sizes), the TinyLFU sketch (aging rate, increment count and the strict admission test are linked and should be tuned together), and the original admit-or-drop baseline, which has been superseded.


---

- finished: 2026-10-29T09:03:21+00:00
- duration: 25.38s
- status: ok
- input_tokens: 4
- output_tokens: 2352
- cache_read_tokens: 41137
- cache_write_tokens: 12912
- cost_usd: 0.158579
