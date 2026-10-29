# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-29T08:55:58+00:00

## timeline

### 08:56:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Merging separate winners has stacked additively every cycle (0002–0006): residency structure, sketch storage, link compaction, conservative update and window sizing combined without cancelling. Merging the previous cycle's winners is a reliable first move.\n- Bit packing pays off over and over: sentinel-tagged links, nibble counters and u32 indices shrank metadata. Once the codegen was fixed (0005), it cost nothing in latency or hit-rate.\n- Cache-line locality in the sketch (blocked layout, fixed row-to-word-pair mapping with no aliasing) fixed the scattered-miss get tail and lets the row RMWs run in parallel.\n- The hit-rate levers split by cache size. The window, the window floor and admission-structure changes help the small size. Conservative update (exact or pairwise) helps the large size by fixing correlated in-block overestimation. Combining them works.\n- An approximate update rule is good enough: pairwise conservative update keeps nearly all of the exact-min accuracy.\n- Recent latency wins have come from codegen and instruction ordering, not from algorithm changes: a single cheap hash, prehashed map keys, `#[inline(always)]` on the increment, u32-native index paths without bounds checks, and prefetching plus putting the probe before the increment.\n\n## Recurring failures\n\n- Hit-rate changes that forget or bypass frequency evidence (aging, adaptive window, uniform window growth) keep slipping slightly at the large size.\n- Runtime adaptivity (the hill-climbing window) added latency on every op, and its gain was confounded by window fills that skipped the frequency contest.\n- Using micro-structure of the update rule as a latency lever did not work: branch-free masks and a shorter min chain recovered nothing. The real cost was the out-of-line call.\n- Rearranging counters inside a fixed sketch block cannot lower load per counter, and doing so hurt put latency. Placement inside the block is a dead end.\n- Tuning the aging period or counter width for the large-size slip was the wrong diagnosis.\n- Per-op-time verdicts from single runs have repeatedly been noise: byte-identical hot paths slowed down, and so did unrelated tails. Latency claims need interleaved or repeated A/B runs.\n\n## Open questions\n\n- Get p99 is the one stubborn metric. Is it caused by `on_hit` probation→protected promotion link misses? Would prefetching or reordering the neighbour nodes fix it?\n- Does the prefetch/probe reorder stack on top of the inlined merged line, or do the two get-p50 gains overlap?\n- Large-size hit-rate is near its ceiling with the current counter layout. Further gains need more counters per addition. Is the small large-size dip on hm_0 a conservative under-counting ceiling or noise?\n- Why is the window-floor gain trace-specific (hm_0 gains, web_0 flat)?\n- The benchmark noise floor is near the size of the effects being measured, so a stable interleaved harness remains a prerequisite.\n- Unmeasured costs remain: per-entry stored hashes and growth of the fixed cache struct are not in the metadata metric.\n\n## Per-policy notes\n\n- W-TinyLFU (window + SLRU main): this is the backbone of every winning line. Size the window as an absolute floor (max of the percentage and a fixed entry count), not as a larger uniform percentage. Do not use an adaptive climber unless the frequency contest is kept.\n- Count-min admission sketch: blocked 64-byte layout, 4-bit packed counters, periodic halving, pairwise conservative update, force-inlined increment. Keep the fixed row-to-word-pair mapping.\n- Residency layer: the 24-byte node with u32 sentinel-encoded links is the base. Its old latency regression was codegen and is now resolved. The earlier hit-rate gap against the 40-byte line came from missing levers, not from `reset()`.\n"
}
```

### 08:56:21 thinking



### 08:56:21 chat

I've written `cross-cycle-insights.md` with the four required sections in order, built only from the six-cycle ledger.

- **Recurring wins:** Merging winners from separate axes has added up every cycle from 0002 to 0006. Packing bits and keeping sketch blocks in one cache line kept paying off. The hit-rate levers split by cache size: the window and window floor help the small size, and conservative update helps the large one. Recent latency gains came from inlining, prefetching and other codegen fixes, not from changing the update rule.
- **Recurring failures:** Mechanisms that forget or skip frequency evidence slip at the large size. The adaptive hill climber cost latency on every op. Trying to speed up the update rule itself (branch-free updates, a shorter min chain) recovered nothing. Moving counters around inside a sketch block hurt rather than helped. Per-op-time verdicts from single runs have repeatedly turned out to be noise.
- **Open questions:** Get p99 is still the one stubborn metric, possibly from link misses when an entry is promoted. It's unknown whether the prefetch gain adds to the inlining gain or overlaps it. The large size seems close to its hit-rate ceiling with the current counter layout. The window-floor gain varies by trace, the benchmark is noisy, and some memory costs aren't counted in the metadata figures.
- **Per-policy notes:** These cover the window/SLRU structure, the admission sketch, and the 24-byte u32-link residency layer, which is now the base line.


---

- finished: 2026-10-29T08:56:21+00:00
- duration: 23.35s
- status: ok
- input_tokens: 4
- output_tokens: 1972
- cache_read_tokens: 40067
- cache_write_tokens: 11597
- cost_usd: 0.140245
