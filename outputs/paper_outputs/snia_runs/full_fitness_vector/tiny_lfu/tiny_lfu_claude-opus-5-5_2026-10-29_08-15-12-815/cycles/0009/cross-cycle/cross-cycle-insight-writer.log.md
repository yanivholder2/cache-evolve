# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-29T09:19:00+00:00

## timeline

### 09:19:19 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0009)\n\n## Recurring wins\n\n- Merging orthogonal winners keeps adding up. Fast hash + W-TinyLFU (0002), aging sketch + window/SLRU (0003), the sketch on the 24-byte node (0004), and window floor + pairwise conservative update + u32 links (0006) all stacked without interference. Merges stop adding up only when two changes act on the same mechanism (see failures).\n- Bit/word packing that removes stores or memory pays off. Sentinel-tagged links, nibble-packed 4-bit counters and the cache-line-blocked sketch each improved metadata and latency together (0002–0003). This holds only when the packed field isn't written on the hot path (see failures).\n- Conservative update is the large-size accuracy lever (0003–0005). It fixes correlated in-block overestimation, and the pairwise approximation keeps nearly all of the gain. An exact global min is unnecessary.\n- An absolute window floor is the small-size lever (0004–0006). It leaves the large size untouched. Small and large sizes keep wanting opposite things, so levers that touch only one size are the safe kind.\n- Latency gains now come from codegen, not algorithm changes. These include u32-only index arithmetic with no bounds checks and single-store links (0005), and `#[inline(always)]` on `increment_hashed` (0006). The supposed conservative-update latency cost was an out-of-line-call artifact.\n- A small, explicit, bounded bonus for re-seen candidates works: the `>= 2`-guarded tie-break on the dedup line (0008 v002). It is the current hit-rate base.\n\n## Recurring failures\n\n- Explaining get p99 as a pointer-chase or link-write cost has failed four times: sketch prefetch, neighbour prefetch, an in-link CLOCK bit, and a bitset CLOCK bit (0007–0009). Protected-segment CLOCK also hurts hit-rate. Drop that line of attack and look at the inlined sketch update and the map probe instead.\n- Software prefetch on the inlined merged line overlaps with the inlining gain instead of adding to it (0007).\n- Changing the sketch layout cannot fix overestimation. In-block row addressing gained nothing (0006), and the fixed row-to-word-pair layout is valuable because its rows never alias.\n- Latency fixes that only remove branches or shorten the `min` chain did nothing (0004–0005).\n- Stacking candidate bonuses doesn't work. The double count, the tie-break and the restored aging clock all fix the same large-size newcomer loss. Combining any two adds nothing at the large size and over-favours recency at the small size (0009).\n- Uniform knobs fail because the two sizes pull in opposite directions. A uniform larger window percentage (0004) and a uniform `tick()` (0009) each helped one size and hurt the other.\n- Runtime hill-climbing window adaptivity (0002) cost latency on every op for a gain only at the small size.\n- Flags packed into link words turn blind relink stores into load-then-store sequences (0008).\n\n## Open questions\n\n- Does a size-dependent aging rate (slower at small capacity) keep the small-size gain while helping the large size? The small-size gain since 0008 traces back to slower aging, not to the tie-break.\n- Should the tie-break threshold scale with capacity or sketch width instead of being a fixed constant?\n- What actually drives get p99? Profile the inlined sketch update and the map probe.\n- How large is the per-op-time noise floor? Single-run latency verdicts flipped on byte-identical hot paths in 0004, 0005 and 0007–0009. Judge latency only from interleaved or repeated A/B runs.\n- Do the gains generalise across traces? The window floor's gain was hm_0-specific, and web_0 is the most sensitive cell at the large size.\n\n## Per-policy notes\n\n- W-TinyLFU window + SLRU: this is the residency backbone. The 24-byte u32-link node is now as fast as the 40-byte one or faster, so build on it. Keep protected as a true LRU (move-to-tail), not CLOCK. Size the window with an absolute floor, not a percentage, and don't use runtime adaptivity.\n- Count-min / frequency sketch: use the cache-line-blocked layout with 4-bit counters, a fixed row-to-word-pair layout, pairwise conservative update and forced-inline increments. Don't tune `SAMPLE_FACTOR` or counter width uniformly. Any change to how often counts are added also changes the reset rate, so the two are coupled.\n- Admission: use the dedup line (no get→put double count, no uniform tick) with a `>= 2`-guarded tie-break. The old double count was an implicit recency bonus. Use only one candidate bonus at a time.\n- CLOCK / reference-bit variants: both in-link and side-bitset forms regressed latency and hit-rate. Drop them.\n"
}
```

### 09:19:27 thinking



### 09:19:27 chat

I've written `cross-cycle-insights.md` in the cycle 0009 cross-cycle directory. It covers all nine cycles under the four required sections, in order, and contains no numbers.

The main patterns it records:

- **Recurring wins:**
  - Merging winners that change separate mechanisms keeps adding up.
  - Conservative update helps the large cache size; the pairwise approximation keeps nearly all of the gain.
  - An absolute window floor helps the small cache size.
  - Latency gains now come from code generation: forced inlining and u32-only indexing.
  - The best hit-rate base is cycle 0008 v002: miss dedup plus a tie-break that admits re-seen keys (`>= 2`).
- **Recurring failures:**
  - Get p99 has resisted four attempts that blamed pointer chasing or link writes.
  - Prefetching overlaps with the inlining gain instead of adding to it.
  - Moving counters around inside a sketch block gains nothing.
  - Stacking more than one bonus for new keys adds nothing at the large size and over-favours recency at the small size.
  - Settings applied the same way at every cache size help one size and hurt the other.
- **Open questions:**
  - An aging rate that depends on cache size, since the small-size gain comes from slower aging rather than the tie-break.
  - A tie-break threshold that scales with capacity or sketch width.
  - Profiling the sketch update and the map probe for get p99.
  - How large the latency noise is between runs.
  - Whether gains hold across traces, since some have only shown up on hm_0.
- **Per-policy notes:** grouped into W-TinyLFU window + SLRU, the frequency sketch, admission, and CLOCK/reference-bit variants.


---

- finished: 2026-10-29T09:19:27+00:00
- duration: 27.29s
- status: ok
- input_tokens: 4
- output_tokens: 2398
- cache_read_tokens: 43141
- cache_write_tokens: 15014
- cost_usd: 0.176716
