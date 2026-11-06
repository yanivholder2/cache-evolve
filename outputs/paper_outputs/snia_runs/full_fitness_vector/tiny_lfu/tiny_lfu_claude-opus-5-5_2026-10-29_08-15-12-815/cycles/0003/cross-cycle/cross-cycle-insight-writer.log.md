# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-29T08:35:15+00:00

## timeline

### 08:35:31 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Merges of winners that each change one part of the cache keep adding up (cycles 0002 and 0003). Fast hash plus window/SLRU, window/SLRU plus the blocked aging sketch, and compact links all combined with no interference on residency. Merging the previous cycle's winners first is a dependable opening move.\n- Bit packing and sentinel encoding keep paying off: tagged links removed the region byte, nibble-packed counters shrank the sketch and made aging cheap, and u32 links cut metadata. Each kept hit-rates the same or better while lowering metadata.\n- Memory-locality fixes deal with tail latency. Single-hash keys, a prehashed map key, and the cache-line-blocked sketch together resolved the get-tail weakness flagged in cycle 0001.\n- Fixes to admission and frequency quality give most of their hit-rate gain at the small cache size. Window/SLRU, aging, and conservative update each improved the small cache most.\n- Conservative update is the clearest causal fix for the large-cache slip. That slip came from correlated in-block overestimation, not from the aging period or counter width.\n\n## Recurring failures\n\n- Per-access work in `get` keeps costing median latency. Examples are u32-counter aging (cycle 0001), hill-climber bookkeeping (cycle 0002), and the saturation and sample checks plus branchy two-pass conservative update (cycle 0003). The bottleneck is control flow, not memory.\n- Runtime adaptivity (the hill-climbing window) cost latency on every op. Its small-cache gain may have come from skipping the frequency contest, not from the window size itself. Shelved.\n- Tuning `SAMPLE_FACTOR` or counter width for the large-cache slip is a dead end, because conservative update already explains and fixes it.\n- Mechanisms that forget or bypass frequency evidence (aging, window fill without a contest) tend to slip a little at large capacity.\n\n## Open questions\n\n- Can a branchless, masked, word-level saturating conservative update keep its accuracy at close to single-pass increment latency?\n- Does conservative update still give its large-cache gain on top of window/SLRU, or does the window absorb part of it?\n- Is the latency regression from compact u32 links real, for example from codegen or bounds checks, or is it noise? It needs a back-to-back, same-session benchmark.\n- Costs that are not yet measured: the per-entry stored hash in the map and the growth of the fixed cache struct. Neither appears in the metadata metric.\n- Does the merged cache still leave a gap at the small size that could justify bringing back adaptivity with the frequency contest restored?\n\n## Per-policy notes\n\n- **W-TinyLFU (window + SLRU):** This has been the dominant residency structure since cycle 0001. It is stable under merges, and its hit-rate gain is concentrated at the small size. Sentinel-tagged and u32 links keep its metadata low.\n- **Count-min / frequency sketch:** It has evolved from a SipHash multi-hash with u32 counters to a single cheap hash, then a cache-line-blocked layout, packed 4-bit counters, periodic halving, and conservative update. Accuracy problems are now solved. What remains is the increment's per-op cost.\n- **Adaptive window (hill climber):** It tested negative once. Revisit only with the contest restored, a step floor, and evidence of a remaining small-cache gap.\n- **Baseline strict admit-or-drop without aging:** This was the original source of hit-rate loss at small capacity and has been fully superseded.\n"
}
```

### 08:35:36 chat

I wrote `cross-cycle-insights.md` in the cycle 0003 cross-cycle directory, built only from the three ledger entries. It has the four required sections in order, with short qualitative bullets and no numbers.

**Recurring wins**
- Merging winners that each change a different part of the cache keeps adding up cleanly.
- Packing bits and encoding links with sentinels lowers metadata without hurting hit-rate.
- Fixes that keep memory access local (one hash, a prehashed map key, a cache-line-blocked sketch) fixed the slow `get` tail.
- Conservative update is the clearest cause-and-effect fix for the hit-rate slip at the large cache size.

**Recurring failures**
- Extra work on every `get` keeps costing median latency.
- The adaptive window (hill climber) didn't pay off and is shelved.
- Tuning the aging sample period or counter width for the large-cache slip is a dead end, because conservative update already explains and fixes it.

**Open questions**
- Can conservative update be made branchless without losing its accuracy?
- Does it still help at the large size once the window/SLRU structure is in place?
- Is the slowdown seen with the compact links real, or benchmark noise?
- Some costs aren't measured yet, such as the hash stored in each map entry and the larger cache struct.

**Per-policy notes** cover the window/SLRU structure, the frequency sketch, the adaptive window and the original baseline policy.


---

- finished: 2026-10-29T08:35:37+00:00
- duration: 21.14s
- status: ok
- input_tokens: 4
- output_tokens: 1818
- cache_read_tokens: 36947
- cache_write_tokens: 8348
- cost_usd: 0.110549
