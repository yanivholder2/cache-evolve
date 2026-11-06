# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-29T09:28:06+00:00

## timeline

### 09:28:39 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Merges of levers that touch separate layers keep adding up with no interference (cycles 0002–0006, 0010). Examples: window/SLRU with fast hash, the aging sketch with residency, u32 links with the blocked sketch, and the window floor with pairwise conservative update. When two winners change different parts of the cache, merge them first.\n- A safe hit-rate lever moves only one cache size. Conservative update and capacity-gated slower aging are clean examples. Levers that act on both sizes tend to help one and hurt the other.\n- Small-size hit-rate responds to admission and recency levers: a larger window or window floor, and slower aging (cycles 0004, 0005, 0008–0010). Slower aging is the confirmed lever and has not saturated yet.\n- Large-size hit-rate responds to the quality of frequency evidence. Conservative update fixed correlated in-block overestimation (cycles 0003–0004). The pairwise min keeps nearly all of its accuracy (cycle 0005).\n- Packing bits into compact metadata paid off: nibble counters, sentinel-tagged u32 links and a 24-byte node (cycles 0002–0005). The latency cost of the compact layout went away once the codegen was fixed.\n- The real latency wins came from codegen and data structures, not from policy changes. Examples: a single cheap hash, the cache-line-blocked sketch, removing bounds checks, `#[inline(always)]` on the increment, and the packed linear-probing index (cycles 0001, 0002, 0005, 0006, 0010).\n- Splitting a coupled change into its separate variables made the cause clear (cycle 0008). The miss double-count turned out to act as an implicit \"seen again\" bonus.\n\n## Recurring failures\n\n- Software prefetch hints, on the sketch or on list neighbours, add nothing once the increment is inlined. The node has to be loaded before its neighbours are known, so prefetch cannot get ahead of the pointer chase (cycles 0006–0007).\n- Explaining the get tail as a list-link or relink cost failed four times: neighbour prefetch, a CLOCK flag in `next`, a CLOCK reference bit in a separate bitset, and the promotion path. Bits packed into link words turn blind stores into load-then-store. Drop CLOCK and reference-bit replacements for protected LRU.\n- Bonuses for the window candidate overlap instead of adding up. The double count, the tie-break and the restored tick all fix the same large-size newcomer loss. Stacking any two of them over-favours recency at the small size (cycle 0009).\n- Making the victim colder works like a candidate bonus: it loosens the filter. Under LRU probation, \"colder by estimate\" mostly means \"young and not yet re-referenced\" (cycle 0010).\n- These diagnoses were wrong or gave nothing: branch removal alone, shortening the `min` chain, uniform window-percentage increases, in-block counter re-addressing, uniform ticks, and runtime hill-climbing windows.\n- Single-run per-op-time verdicts are unreliable. Byte-identical hot paths swung both ways in at least five cycles (0004, 0005, 0007–0010). Read hit-rate and latency verdicts separately, and trust a latency result only when it is consistent across all four latency stats or has a clear mechanism.\n\n## Open questions\n\n- Where is the ceiling on small-size aging slowdown? 4-bit counter saturation is the suspected limit. Would a mild slowdown at the large size also help there?\n- Can put p99 under the packed index be fixed, through a fused remove+insert on eviction or a lower load factor, without losing the get gain? Does the index gain hold up in interleaved runs and stack with the aging change?\n- Should the tie-break threshold, the aging factor and the window floor scale with capacity or sketch width instead of being constants?\n- Would an age-aware or stale-frequency victim signal succeed where raw estimate comparison failed?\n- What is left of get p99 after the index change? The inlined sketch update is the main remaining suspect.\n- Can the benchmark be made stable with interleaved A/B and repeat runs, so that latency effects near the noise floor can be judged?\n\n## Per-policy notes\n\n- **W-TinyLFU (window + SLRU), the current line:** The base is the 24-byte u32-link node, the blocked 4-bit aging sketch, pairwise conservative update with forced inlining, a size-aware window floor, miss dedup with no tick, a `>= 2` tie-break, and capacity-gated slower aging. The packed index is pending merge. Keep the protected segment as strict LRU.\n- **TinyLFU sketch:** Keep the cache-line-blocked layout with fixed row-to-word-pairs, since rows that never alias let the updates run independently. Fight overestimation with conservative update, not with placement inside the block. Any change to how often increments are added also changes the aging clock and tie behaviour, so keep the two coupled in mind.\n- **Admission contest:** The strict `>` test underserves re-seen newcomers. Use one explicit, bounded bonus, not several.\n- **Adaptive window (hill climber):** It cost latency on every op, and its gain may have come from bypassing the frequency contest. Revisit only with the contest restored and a step floor.\n- **Map/index:** The hashbrown index was a hidden latency cost. Linear probing helps get, and its eviction-path deletion is the new put-tail cost.\n"
}
```

### 09:28:45 chat

I wrote `cross-cycle-insights.md` in the cycle 0010 cross-cycle output folder. It's a fresh summary of all ten cycles in the ledger, using the four required sections in order.

- **Recurring wins:** Changes that touch separate parts of the cache keep adding up when merged. The safe hit-rate levers each move only one cache size: slower aging helps the small size, and conservative update helps the large size. The real latency gains came from code generation and data structures, such as forced inlining and the packed index, not from policy changes.
- **Recurring failures:** Prefetch hints stopped helping once the sketch increment was inlined. Four attempts to explain slow `get` tails as a linked-list cost all failed. Bonuses for new cache entries overlap instead of adding up, and picking a colder eviction victim has the same effect as a bonus. Single-run latency results swung both ways on unchanged code in at least five cycles, so they can't be trusted on their own.
- **Open questions:** How far slower aging can go before 4-bit counters saturate. Whether the packed index's slower `put` tail can be fixed without losing its `get` gain. Whether the tie-break threshold, aging rate and window floor should scale with capacity. What still causes the slow `get` tail.
- **Per-policy notes:** The current base setup is listed in full, with notes for the sketch, the admission test, the adaptive window and the map index.

All numbers are kept in qualitative terms; none were added beyond what the ledger states.


---

- finished: 2026-10-29T09:28:45+00:00
- duration: 39.17s
- status: ok
- input_tokens: 4
- output_tokens: 2474
- cache_read_tokens: 44220
- cache_write_tokens: 16270
- cost_usd: 0.1885
