# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:57:09+00:00

## timeline

### 15:57:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Cutting per-op hashing cost on the baseline lineage pays: single-hash sketch (c0001), cheaper index derivation (c0002), cheap map hasher (c0004), and the flat table with one hash reused for the sketch (c0009, the largest win). The map and hashing path is the dominant per-op cost.\n- Storage-layer changes (map hasher, flat table over a slab) move latency while leaving hit-rate and policy untouched. These are the safest speed gains.\n- Gains tend to appear on the baseline and then fail to transfer to stronger lineages (c0005 to c0006). Test stacking explicitly.\n\n## Recurring failures\n\n- Hit-rate levers on workload-a have all been flat or negative: aging (twice), small and smaller windows, victim sampling, tie-admitting, doorkeeper, wider sketch, conservative update, segmented main LRU. The baseline's strict-greater admission with the LRU head as victim looks near the ceiling.\n- Sketch-internal tweaks (width, accuracy, early-exit, hash derivation) are exhausted. Sketch accuracy is saturated.\n- Footprint cuts (compact Node, narrower or halved sketch, field removal) usually do not become speed gains. They often add conversion or unpacking cost, and alignment can absorb the saving. The working set is small, so locality does not help.\n- Extra hot-path state or bookkeeping (doorkeeper bitset, victim cache, cached fields, inline hints) raises tail latency and struct size.\n- Pre-sizing, inline and branch hints, and relink skipping did nothing. Allocation and list relinking are not the cost.\n- Sketch heap changes are invisible in the headline metadata number, which tracks Node size and is dominated by the sketch.\n\n## Open questions\n\n- Does the c0009 flat-table and single-hash gain stack on the best hit-rate incumbents? Which part carries it: the table or the hash reuse?\n- Is any hit-rate lever left on workload-a? The adaptive hill-climbing window was never evaluated because of an implementer failure.\n- Is there a profile of where get and put spend time? Most hot-path guesses have been blind.\n- How much latency delta is noise? Repeat runs are needed before crediting small deltas, and several earlier \"gains\" were likely noise.\n\n## Per-policy notes\n\n- tiny_lfu (window + sketch admission + LRU main): sketch admission already gives the scan resistance this trace needs. Hit-rate is robust to every structural and sketch variation tried.\n- Winner status often reflects the selection rule, not real improvement. Several kept variants were regressions or flat.\n- Several insights were written without source diffs, so some attributions are inferred. Audit diffs for unexplained size or latency changes before building on a variant (e.g. c0008 v002).\n- Next direction: port the c0009 flat-table storage onto the best incumbent, and target the map and hashing path rather than the sketch or list.\n"
}
```

### 15:57:24 chat

I wrote `cross-cycle-insights.md` in the cycle's `cross-cycle` output directory, with the four fixed sections in order. I built it only from the cycle 0001–0009 ledger digest, without reading the previous version.

- **Recurring wins:** Cutting per-op hashing cost has paid off repeatedly, and the cycle 0009 flat-table change was the biggest win. Storage-layer changes are the safest speed gains because hit-rate stays untouched. Gains found on the baseline often fail to carry over to stronger lineages.
- **Recurring failures:**
  - Every hit-rate lever tried on workload-a has been flat or negative.
  - Sketch-internal tweaks are exhausted.
  - Footprint cuts rarely become speed gains.
  - Extra hot-path state raises tail latency.
- **Open questions:**
  - Does the flat-table gain stack on the best incumbents, and which part of it carries the gain?
  - Is any hit-rate lever left? The adaptive window was never tested.
  - Where does time actually go in `get` and `put`?
  - How much of the latency movement is noise?
- **Per-policy notes:** These cover tiny_lfu admission, the fact that some Winners only passed the selection rule, the inferred attributions where no source diff was available, and the next direction (port the flat table onto the best incumbent).


---

- finished: 2026-10-20T15:57:25+00:00
- duration: 15.78s
- status: ok
- input_tokens: 4
- output_tokens: 1585
- cache_read_tokens: 41190
- cache_write_tokens: 12868
- cost_usd: 0.072339
