# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:32:34+00:00

## timeline

### 15:32:43 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_15-21-40-321/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Making the sketch cheaper (fewer hashes, single mix, bit-sliced indexes, smaller counters) is the most reliable gain. It cuts per-op time, especially the put tail, and shrinks the sketch.\n- Changes that leave counters, conservative update and admission untouched keep hit-rate flat, so they are safe speed wins.\n- Layout shrinks (u32 indices, compact sketch) reduce footprint without hurting hit-rate.\n\n## Recurring failures\n\n- Aging or decay of any period fails on workload-a. Its frequencies are stable, so decay discards useful history. This has failed in several forms.\n- Extra admission mechanism (small window, victim sampling, tie-admitting `>=`) adds cost and gives no hit-rate gain. Admitting ties actively lowers hit-rate because ties are common at low frequencies.\n- Cutting sketch precision too far (4-bit counters, one row, tight sizing) buys speed and footprint but loses hit-rate.\n- Extra hot-path state or conversions (cached state, NIL checks, index widening) tend to cost latency that outweighs the locality gain on a small working set.\n- Small cache-struct additions grow Node size. The headline metadata number tracks Node size, not sketch heap, so sketch savings are invisible there.\n\n## Open questions\n\n- How can the nibble sketch recover hit-rate with the least added cost (second hash, larger table)? Which factor causes the loss: the 4-bit cap, one row, or the sizing?\n- Do the single-hash speedup, the compact sketch and the u32 layout combine, or do they offset each other?\n- Can the u32 layout's latency cost be recovered (fewer conversions, unchecked indexing), or is it noise?\n- Is there a new signal that separates tied frequencies without hot-path work, such as a doorkeeper?\n- How large is latency noise? Small time deltas need repeated or larger runs before they are trusted.\n- Which parts of the u8/single-hash/conservative-update rewrite account for its gains?\n- Is there any drifting trace that would justify decay or a larger window?\n\n## Per-policy notes\n\n- TinyLFU (strict greater-than admission, LRU head as victim, never-decaying counters): this baseline fits workload-a well. Gains come from sketch cost and accuracy, not from changes to admission comparison or victim choice.\n- The rewritten single-hash u8 conservative-update sketch is the established base. The single-mix hash variant is the preferred speed parent because it has no hit-rate cost.\n- The compact u32-index layout is the footprint parent, with a latency cost still to win back.\n- Several insights were written without source access, so attributions of gains to specific code changes are inferred.\n"
}
```

### 15:33:10 chat

I wrote `cross-cycle-insights.md` to the output dir, with the four required sections in order. I didn't read the previous version, so this is a fresh projection of the three-cycle ledger.

- **Recurring wins:**
  - Making the sketch cheaper (single hash, single-mix indexing, smaller counters) is the most reliable gain.
  - Changes that leave counters, conservative update and admission alone keep hit-rate flat.
  - Layout shrinks cut footprint without hurting hit-rate.
- **Recurring failures:**
  - Aging or decay fails on workload-a because its frequencies are stable.
  - Extra admission mechanism (small window, victim sampling, tie-admitting `>=`) adds cost without a hit-rate gain.
  - Cutting sketch precision too far loses hit-rate.
  - Extra hot-path state or conversions cost latency.
  - Sketch savings don't show in the headline metadata number, which tracks Node size.
- **Open questions:**
  - How to recover hit-rate in the nibble sketch cheaply.
  - Whether the speed, compact-sketch and layout wins combine.
  - Whether the layout's latency cost is real or noise.
  - Whether a doorkeeper could separate tied frequencies.
  - Which parts of the sketch rewrite account for its gains.
  - Whether any drifting trace would justify decay.
- **Per-policy notes:**
  - TinyLFU's strict-greater admission with the LRU head as victim fits workload-a, so gains come from the sketch.
  - The single-hash u8 sketch is the base, and the single-mix hash variant is the preferred speed parent.
  - The u32-index layout is the footprint parent, with its latency cost still to win back.
  - Several insights were written without source access, so attributions to specific code changes are inferred.

I kept everything qualitative and didn't add any numbers.


---

- finished: 2026-10-20T15:33:11+00:00
- duration: 36.73s
- status: ok
- input_tokens: 4
- output_tokens: 1682
- cache_read_tokens: 36081
- cache_write_tokens: 7676
- cost_usd: 0.051517
