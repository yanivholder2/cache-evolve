# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-29T08:48:29+00:00

## timeline

### 08:48:45 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Merging winners from separate axes has added up every time (cycles 2–5). Residency, sketch storage, link layout and update rule have not interfered with each other. Merge first, then add new mechanisms.\n- Packing bits keeps paying off. Sentinel-tagged links, nibble-packed 4-bit counters and u32 indices each cut metadata at little or no hit-rate cost. Once the codegen was fixed, the 24-byte node also costs nothing in latency.\n- Cache-line-blocked sketch layout fixed the scattered-miss get tail (cycle 2). It has held up as the sketch base since.\n- Most hit-rate gains come at the small cache size: window/SLRU, aging, a larger or floored window, and the adaptive window all gain there. The exception is conservative update, the one lever that reliably helps the large size, by fixing correlated in-block overestimation.\n- Approximate conservative update (pairwise min) keeps nearly all the accuracy of the exact min. Exactness in the sketch update rule is not worth paying for.\n\n## Recurring failures\n\n- Trying to cut latency in the sketch increment, whether by removing branches (cycle 4) or shortening the `min` chain (cycle 5), has not recovered per-op-time. The diagnosis that the serial min chain sets latency has been tried and dropped. Stop trying variants of this.\n- Runtime adaptivity (hill-climbing window) cost latency on every op and gained only at the small size. A static size-aware window floor gets that gain more cheaply.\n- Uniform changes to the window percentage, or changes that forget or bypass frequency evidence, keep causing small slips at the large cache size.\n- Tuning `SAMPLE_FACTOR` or counter width was ruled out as the fix for the large-cache slip, because conservative update explains and fixes it.\n- Per-op-time verdicts have kept turning out unreliable (cycles 3–5). Byte-identical hot paths have shown slowdowns, so single-run latency deltas are near the benchmark's noise floor.\n\n## Open questions\n\n- Benchmark stability: do the latency regressions in cycle 5's v002 and v003 survive interleaved, repeated A/B runs against their parents?\n- Does adding conservative update (full or pairwise) stop `increment_hashed` from being inlined into `get`/`put`? Is that the real source of its latency cost?\n- The hit-rate mismatch between the 24-byte and 40-byte lines' sketches (open since cycle 4) needs an answer before the lines are merged.\n- Why is the window floor's small-size gain specific to the trace (hm_0 gains, web_0 barely moves)?\n- Is there any lever left that gains hit-rate at the large size beyond conservative update?\n- Costs not measured yet: the per-entry map hash and growth of the fixed cache struct are missing from the metadata figures.\n\n## Per-policy notes\n\n- W-TinyLFU (window + SLRU main): this is the core line. It has been stable under every merge since cycle 2. The window should be sized as an absolute floor (max of a percentage and an entry count), not a uniform percentage.\n- Count-min / TinyLFU sketch: the blocked 4-bit aging design with a single cheap hash is the settled base. Conservative update (pairwise form) is the preferred update rule. The increment is where per-op cost sits, but the cause looks like codegen or inlining, not arithmetic.\n- Residency layer (linked SLRU nodes): the 24-byte u32-link layout with tagged single-store links and bounds-check-free indexing is now the base. Its earlier slowdown was link codegen and is fixed. The `NIL` sentinel on `next` and a prehashed map key are part of this line.\n- Plain admit-or-drop baseline with a never-aging sketch: superseded since cycle 1. Do not return to it.\n"
}
```

### 08:48:52 chat

I've written `cross-cycle-insights.md` in the cycle 0005 cross-cycle folder. It uses the four required sections and draws only on the ledger for cycles 1–5.

- **Recurring wins:** Merging winners that each changed a different axis has added up cleanly every time from cycle 2 to 5. Packing bits keeps cutting metadata, and the cache-line-blocked sketch fixed the slow `get` tail. Most hit-rate gains come at the small cache size. Conservative update is the one lever that also helps at the large size, and the cheaper pairwise form keeps nearly all of its accuracy.
- **Recurring failures:** Making the sketch increment faster, first by removing branches and then by shortening the `min` chain, never recovered latency. The adaptive hill-climbing window cost latency on every op and gained only at the small size. Raising the window percentage uniformly causes small slips at the large size. Since cycle 3, single-run latency differences have been about the size of the benchmark's noise.
- **Open questions:**
  - Do the cycle 5 slowdowns hold up in repeated, interleaved runs?
  - Does conservative update stop `increment_hashed` from being inlined into `get`/`put`?
  - Why does the 24-byte line's hit-rate differ from the 40-byte line's?
  - Why does the window floor help on hm_0 but barely on web_0?
  - Is there any large-cache lever beyond conservative update?
  - The per-entry map hash and the growth of the cache struct still aren't counted in the metadata figures.
- **Per-policy notes:** These cover the W-TinyLFU window plus SLRU line, the blocked 4-bit aging sketch with pairwise conservative update, and the 24-byte u32-link residency layer. They also mark the original admit-or-drop baseline as superseded.


---

- finished: 2026-10-29T08:48:52+00:00
- duration: 23.33s
- status: ok
- input_tokens: 4
- output_tokens: 2037
- cache_read_tokens: 39011
- cache_write_tokens: 10506
- cost_usd: 0.132606
