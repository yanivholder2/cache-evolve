# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-29T08:41:45+00:00

## timeline

### 08:42:02 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Merges add up. Every merge so far has combined without interference: fast hash with window/SLRU, window/SLRU with the aging sketch, and the aging sketch with compact links. Building one mechanism per axis and then merging has been a reliable way forward.\n- Bit packing keeps paying off. Sentinel-tagged links, 4-bit nibble counters and u32 links each cut metadata without hurting hit-rate. Packed counters also made aging cheap enough that its earlier latency cost went away.\n- Lay memory out for the cache line. Single-hash access and the cache-line-blocked sketch removed scattered misses and fixed the get tail flagged in cycle 0001. That fix has held since.\n- Hit-rate levers split by cache size. Window/SLRU, a larger window and aging gain most at the small size. Conservative update gains at the large size, where small overestimates decide close frequency contests. The two kinds look complementary.\n- Conservative update is the best-supported causal fix in the ledger. The large-cache slip came from correlated in-block overestimation, and conservative update fixed it twice: once on the plain sketch and once on top of window/SLRU.\n\n## Recurring failures\n\n- Every recent cohort has lost per-op-time on the sketch increment. Saturation checks, sample counting, two-pass conservative update and branch-free `min` chains each cost latency. Removing branches alone does not help, because the serial dependency chain is the real cost.\n- Mechanisms that forget or bypass frequency evidence cost hit-rate at the large size. This has happened with aging, with the window/SLRU bypass, with the adaptive window, and with a uniformly larger window percentage.\n- Runtime adaptivity (the hill-climbing window) cost latency on every op and gained only at the small size. Its gain may come from skipping the frequency contest, not from the window size it picked.\n- Per-op-time verdicts are near the noise floor. Two later cycles had unexplained regressions, one with a byte-identical hot path. Single-run latency deltas are not trustworthy evidence.\n- The u32 links bring a real latency regression, confirmed in a same-run comparison. It sits in the residency-layer codegen, not in the sketch.\n\n## Open questions\n\n- Can a SWAR, pairwise or approximate `min` keep conservative update's accuracy at the increment latency the sketch had before conservative update?\n- Does a window sized as an absolute floor (max of a percentage and a fixed entry count) keep the small-size gain without the large-size slip, once combined with conservative update?\n- What causes the u32-link slowdown: bounds checks or mask/widen sequences in unlink, push_back or on_hit?\n- Why does the ported aging sketch on the compact-link layout not reproduce the reference hit-rate exactly: a faulty copy, or a different residency order?\n- How large is the benchmark noise floor? Repeated or interleaved A/B runs are needed before more latency verdicts.\n- Some costs are not measured yet: the per-entry map hash and the growth of the fixed cache struct.\n\n## Per-policy notes\n\n- **Baseline (strict admit-or-drop, non-forgetting sketch, SipHash):** two separate weaknesses, admission for hit-rate and hashing for latency. Both are solved now, so do not go back to it.\n- **W-TinyLFU window + SLRU:** the core residency structure, and the most reliable lever at the small size. A fixed 1% window is too small for recency bursts at the small size. A uniformly larger percentage takes slots from the main region at the large size.\n- **Blocked 4-bit aging sketch:** cheap in memory and in the tail. Its large-size slip comes from in-block collisions, not the aging period, so do not tune the sample period or counter width for it.\n- **Conservative update:** clear hit-rate gain at the large size on both parents. Its cost is entirely increment latency, so the next lever is shortening the critical path.\n- **Adaptive (hill-climbing) window:** set aside. Revisit only with the frequency contest restored, a climber that stops at its step floor, and a remaining small-size gap.\n- **Compact (u32) links:** hit-rate-neutral and a metadata win, with a confirmed latency cost. Fix the codegen separately before relying on them.\n"
}
```

### 08:42:07 chat

I've rebuilt `cross-cycle-insights.md` from the four-cycle ledger alone, with the four sections in the required order and no numbers.

- **Recurring wins:** Merging one-mechanism-per-axis variants has worked every time, with no interference. Packing bits and laying the sketch out by cache line keep cutting memory cost. The hit-rate levers split by cache size: the window helps the small size and conservative update helps the large one. Conservative update has the strongest causal evidence in the ledger.
- **Recurring failures:** The sketch increment has cost latency every cycle since 0003, and its serial `min` chain matters more than its branches. Mechanisms that forget or bypass frequency evidence lose hit-rate at the large size. The adaptive window isn't worth its latency cost. The u32-link slowdown is confirmed. Latency verdicts now sit close to the benchmark's noise floor.
- **Open questions:** A faster `min` that keeps conservative update's accuracy, an absolute-floor window size, the cause of the u32-link slowdown, why the ported sketch's hit-rate doesn't match exactly, how large the benchmark noise is, and the costs the metadata metric doesn't capture.
- **Per-policy notes:** One short entry each for the baseline, window/SLRU, the blocked aging sketch, conservative update, the adaptive window and compact links.


---

- finished: 2026-10-29T08:42:07+00:00
- duration: 22.72s
- status: ok
- input_tokens: 4
- output_tokens: 2022
- cache_read_tokens: 37916
- cache_write_tokens: 9550
- cost_usd: 0.124439
